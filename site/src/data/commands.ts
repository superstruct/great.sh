export const installCommand = 'curl -sSL https://great.sh/install.sh | sh'

export const initWizardOutput = `$ great init

  Welcome to great.sh — AI Dev Environment Manager

  ? What would you like to set up?
  > Full AI Development Environment (recommended)

  ? Select your AI agents:
  > [x] Claude Code (orchestrator)
    [x] OpenAI Codex CLI
    [x] Google Gemini CLI

  ? Select MCP servers to configure:
  > [x] GitHub (repos, PRs, issues)
    [x] Filesystem (secure file access)
    [x] Memory (knowledge graph)
    [x] Playwright (browser automation)

  ? Enter your API keys (stored in system keychain):
    ANTHROPIC_API_KEY: ********************
    OPENAI_API_KEY: ********************

  [check] Installing tools: node 22, python 3.12, gh, ripgrep...
  [check] Configuring Claude Code with 2 AI agent MCP servers
  [check] Configuring 4 additional MCP servers
  [check] Injecting credentials into MCP server configs

  Your AI dev environment is ready!

  Run \`claude\` to start Claude Code with all MCP servers.
  Run \`great loop install\` to add the great.sh Loop agent team.`

export const sampleToml = `# great.toml — AI Dev Environment Specification

[project]
name = "my-saas-app"
template = "ai-fullstack-ts"

[tools]
node = "22"
python = "3.12"

[tools.cli]
packages = [
  "gh", "docker", "ripgrep", "fzf",
  "starship", "zoxide", "bat", "eza",
  "lazygit", "atuin", "zellij",
]

[agents.claude]
role = "orchestrator"

[agents.codex]
role = "mcp-server"
transport = "stdio"

[agents.gemini]
role = "mcp-server"
transport = "stdio"

[mcp.filesystem]
source = "registry:modelcontextprotocol/server-filesystem"
transport = "stdio"

[mcp.memory]
source = "registry:modelcontextprotocol/server-memory"
transport = "stdio"

[mcp-bridge]
preset = "agent"
default-backend = "gemini"

[secrets]
provider = "great-vault"
required = [
  "ANTHROPIC_API_KEY",
  "OPENAI_API_KEY",
  "GITHUB_TOKEN",
]`

export const loopInstallOutput = `$ great loop install --project

  great.sh Loop -- Installing plugin

  [check] Marketplace registered
  [check] Plugin installed via claude plugin install
  [check] Agent Teams config -> ~/.claude/teams/loop/
  [check] Settings updated (env, statusLine) -> ~/.claude/settings.json
  [check] .tasks/ created, .gitignore updated

  great.sh Loop installed!

  Roles: builder, verifier, reviewer + optional scout
  Model: inherits your session model (pin per-role in teams config)
  Usage: claude -> /great:loop [task description]`

export const bridgeAddCommand =
  'claude mcp add --scope user great-bridge -- great mcp-bridge'

export const bridgeDemoOutput = `$ claude
> Get a second opinion on this diff: ask gemini to review it,
  and ask the local ollama model to summarize the risk.

* great-bridge - prompt (backend: "gemini")
  |- Gemini: Rename is safe, but line 42 drops the error
     context — propagate the source error instead.

* great-bridge - prompt (backend: "ollama", model: "llama3.2")
  |- Ollama: Low risk. One behavioral change in retry logic.

* Both backends agree the rename is safe. Fixing line 42.`

export const mcpBridgeOutput = `$ great mcp-bridge --preset agent

  great.sh MCP Bridge — Starting (preset: agent)

  Discovering backends...
  [check] Gemini CLI    gemini (GEMINI_API_KEY set)
  [check] Codex CLI     codex  (OPENAI_API_KEY set)
  [check] Claude CLI    claude (logged in)
  [check] Ollama        ollama (local)

  Preset: agent (7 tools)
  Tools: prompt, run, wait, list_tasks, get_result,
         kill_task, cleanup_tasks

  Listening on stdio (JSON-RPC 2.0)`
