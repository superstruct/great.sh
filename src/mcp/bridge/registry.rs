use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::process::Command;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::backends::{build_command_args, BackendConfig};

/// The lifecycle state of a background task.
#[derive(Debug, Clone)]
pub enum TaskState {
    Running {
        pid: u32,
        started_at: Instant,
    },
    Completed {
        exit_code: i32,
        stdout: String,
        stderr: String,
        duration: Duration,
    },
    Failed {
        error: String,
        duration: Duration,
    },
    TimedOut {
        duration: Duration,
    },
    Killed,
}

/// A non-owning snapshot of a task for reporting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub backend: String,
    pub status: String,
    pub prompt_preview: String,
    #[serde(skip)]
    #[allow(dead_code)] // Planned for MCP client-facing duration reporting.
    pub started_at: Option<Instant>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Internal task handle stored in the registry.
struct TaskHandle {
    task_id: String,
    backend: String,
    prompt_preview: String,
    state: TaskState,
    /// When the task reached a terminal state (for cleanup timing).
    completed_at: Option<Instant>,
}

/// Thread-safe registry of spawned backend processes.
///
/// All methods take `&self` (shared reference) because internal state is
/// behind `Arc<Mutex<...>>`.
#[derive(Clone)]
pub struct TaskRegistry {
    tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
    pub default_timeout: Duration,
    pub auto_approve: bool,
}

impl TaskRegistry {
    pub fn new(timeout_secs: u64, auto_approve: bool) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            default_timeout: Duration::from_secs(timeout_secs),
            auto_approve,
        }
    }

    /// Spawn a backend CLI process asynchronously. Returns the task ID.
    ///
    /// The process is spawned with `kill_on_drop(true)` so that dropping the
    /// child handle kills the process. A background tokio task collects the
    /// output and updates the registry.
    pub async fn spawn_task(
        &self,
        backend: &BackendConfig,
        prompt: &str,
        timeout_override: Option<Duration>,
        model_override: Option<&str>,
        system_prompt: Option<&str>,
    ) -> anyhow::Result<String> {
        let task_id = Uuid::new_v4().to_string();
        let timeout = timeout_override.unwrap_or(self.default_timeout);

        let (binary, args) = build_command_args(
            backend,
            prompt,
            model_override,
            system_prompt,
            self.auto_approve,
        );

        let mut cmd = Command::new(&binary);
        cmd.args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Create new process group on Unix so we can kill the tree
        #[cfg(unix)]
        {
            // SAFETY: setpgid(0, 0) is async-signal-safe (POSIX) and is called
            // between fork() and exec(). It places the child in its own process
            // group so that killpg can terminate the entire tree on shutdown.
            unsafe {
                cmd.pre_exec(|| {
                    if libc::setpgid(0, 0) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }

        let child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn {} ({}): {}", backend.name, binary, e))?;

        let pid = child.id().unwrap_or(0);

        let handle = TaskHandle {
            task_id: task_id.clone(),
            backend: backend.name.to_string(),
            prompt_preview: prompt.chars().take(80).collect(),
            state: TaskState::Running {
                pid,
                started_at: Instant::now(),
            },
            completed_at: None,
        };

        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(task_id.clone(), handle);
        }

        // Background task to collect output
        let tasks_ref = self.tasks.clone();
        let tid = task_id.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            let result = tokio::time::timeout(timeout, child.wait_with_output()).await;
            let duration = start.elapsed();

            let new_state = match result {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let exit_code = output.status.code().unwrap_or(-1);
                    TaskState::Completed {
                        exit_code,
                        stdout,
                        stderr,
                        duration,
                    }
                }
                Ok(Err(e)) => TaskState::Failed {
                    error: e.to_string(),
                    duration,
                },
                Err(_) => {
                    // Timeout -- the child is killed on drop
                    TaskState::TimedOut { duration }
                }
            };

            let mut tasks = tasks_ref.lock().await;
            if let Some(handle) = tasks.get_mut(&tid) {
                handle.state = new_state;
                handle.completed_at = Some(Instant::now());
            }
        });

        Ok(task_id)
    }

    /// Get a snapshot of a task's current state.
    pub async fn get_task(&self, task_id: &str) -> Option<TaskSnapshot> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).map(|h| self.snapshot(h))
    }

    /// List all tasks (triggers cleanup of old completed tasks).
    pub async fn list_tasks(&self) -> Vec<TaskSnapshot> {
        let mut tasks = self.tasks.lock().await;

        // Cleanup terminal-state tasks older than 30 minutes
        let cutoff = Instant::now() - Duration::from_secs(30 * 60);
        tasks.retain(|_, h| h.completed_at.is_none_or(|t| t > cutoff));

        tasks.values().map(|h| self.snapshot(h)).collect()
    }

    /// Kill a running task.
    pub async fn kill_task(&self, task_id: &str) -> anyhow::Result<()> {
        let mut tasks = self.tasks.lock().await;
        let handle = tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("task {} not found", task_id))?;

        match &handle.state {
            TaskState::Running { pid, .. } => {
                let pid = *pid;
                // Kill process group on Unix (guard: pid 0 would signal our own group)
                #[cfg(unix)]
                if pid > 0 {
                    // SAFETY: killpg sends a signal to the entire process group.
                    // The pid was obtained from a process we spawned and placed
                    // in its own group via setpgid(0, 0). SIGTERM allows graceful
                    // shutdown; SIGKILL after 2s ensures cleanup.
                    unsafe {
                        libc::killpg(pid as libc::pid_t, libc::SIGTERM);
                    }
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        // SAFETY: Same process group as above — fallback SIGKILL.
                        unsafe {
                            libc::killpg(pid as libc::pid_t, libc::SIGKILL);
                        }
                    });
                }
                handle.state = TaskState::Killed;
                handle.completed_at = Some(Instant::now());
                Ok(())
            }
            _ => anyhow::bail!(
                "task {} is not running (state: {:?})",
                task_id,
                handle.state
            ),
        }
    }

    /// Kill all running tasks. Called on server shutdown.
    pub async fn shutdown_all(&self) {
        let mut tasks = self.tasks.lock().await;
        for handle in tasks.values_mut() {
            if let TaskState::Running { pid, .. } = &handle.state {
                #[cfg(unix)]
                if *pid > 0 {
                    // SAFETY: Sends SIGKILL to the process group we created.
                    // Called during server shutdown to ensure no orphaned processes.
                    unsafe {
                        libc::killpg(*pid as libc::pid_t, libc::SIGKILL);
                    }
                }
                handle.state = TaskState::Killed;
                handle.completed_at = Some(Instant::now());
            }
        }
    }

    /// Wait for specific tasks to reach terminal state.
    pub async fn wait_for_tasks(
        &self,
        task_ids: &[String],
        timeout: Duration,
    ) -> Vec<TaskSnapshot> {
        let deadline = Instant::now() + timeout;
        loop {
            let tasks = self.tasks.lock().await;
            let all_done = task_ids.iter().all(|id| {
                tasks
                    .get(id)
                    .is_none_or(|h| !matches!(h.state, TaskState::Running { .. }))
            });
            if all_done || Instant::now() >= deadline {
                return task_ids
                    .iter()
                    .filter_map(|id| tasks.get(id).map(|h| self.snapshot(h)))
                    .collect();
            }
            drop(tasks); // release lock before sleeping
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn snapshot(&self, handle: &TaskHandle) -> TaskSnapshot {
        let (status, exit_code, stdout, stderr, duration_ms) = match &handle.state {
            TaskState::Running { .. } => ("running".to_string(), None, None, None, None),
            TaskState::Completed {
                exit_code,
                stdout,
                stderr,
                duration,
                ..
            } => (
                "completed".to_string(),
                Some(*exit_code),
                Some(stdout.clone()),
                Some(stderr.clone()),
                Some(duration.as_millis() as u64),
            ),
            TaskState::Failed { error, duration } => (
                format!("failed: {}", error),
                None,
                None,
                Some(error.clone()),
                Some(duration.as_millis() as u64),
            ),
            TaskState::TimedOut { duration } => (
                "timed_out".to_string(),
                None,
                None,
                None,
                Some(duration.as_millis() as u64),
            ),
            TaskState::Killed => ("killed".to_string(), None, None, None, None),
        };

        TaskSnapshot {
            task_id: handle.task_id.clone(),
            backend: handle.backend.clone(),
            status,
            prompt_preview: handle.prompt_preview.clone(),
            started_at: match &handle.state {
                TaskState::Running { started_at, .. } => Some(*started_at),
                _ => None,
            },
            exit_code,
            stdout,
            stderr,
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_registry_is_empty() {
        let registry = TaskRegistry::new(300, true);
        let tasks = registry.list_tasks().await;
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_snapshot_serializes_to_json() {
        let snapshot = TaskSnapshot {
            task_id: "test-id".to_string(),
            backend: "claude".to_string(),
            status: "running".to_string(),
            prompt_preview: "hello".to_string(),
            started_at: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            duration_ms: None,
        };
        let json = serde_json::to_string(&snapshot);
        assert!(json.is_ok());
        let json_str = json.expect("serialization should succeed");
        assert!(json_str.contains("test-id"));
    }
}
