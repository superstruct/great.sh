//! Per-backend structured output parsers.
//!
//! Each AI CLI backend produces output in a different format when structured
//! output flags are enabled. These parsers extract metadata (session IDs,
//! token counts, tool usage) from the raw output, falling back to raw text
//! when parsing fails.

use serde::{Deserialize, Serialize};

/// Parsed output from a backend CLI invocation.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedOutput {
    /// Final response text (extracted from structured output, or raw text).
    pub result: String,
    /// Backend session/thread ID for multi-turn continuation.
    pub session_id: Option<String>,
    /// Token usage statistics.
    pub usage: Option<TokenUsage>,
    /// Tools the backend invoked during this request.
    pub tool_usage: Vec<ToolUsageEntry>,
    /// Whether JSON parsing succeeded (false = raw text fallback).
    pub is_structured: bool,
}

/// Token usage statistics from a backend response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// A single tool invocation recorded by the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsageEntry {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

/// Top-level dispatch: parse output based on backend name.
pub fn parse_output(backend_name: &str, raw: &str) -> ParsedOutput {
    match backend_name {
        "claude" => parse_claude_output(raw),
        "codex" => parse_codex_output(raw),
        "gemini" => parse_gemini_output(raw),
        _ => parse_raw_output(raw),
    }
}

/// Parse Claude CLI stream-json output (NDJSON).
///
/// Expected line types:
/// - `{"type":"system","session_id":"..."}` — session metadata
/// - `{"type":"result","result":"...","usage":{...}}` — final result
/// - `{"type":"assistant","message":{"content":[{"type":"tool_use","name":"...","input":{...}}]}}` — tool calls
fn parse_claude_output(raw: &str) -> ParsedOutput {
    let mut session_id: Option<String> = None;
    let mut result_text = String::new();
    let mut usage: Option<TokenUsage> = None;
    let mut tool_usage = Vec::new();
    let mut any_parsed = false;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        any_parsed = true;

        match val.get("type").and_then(|t| t.as_str()) {
            Some("system") => {
                if let Some(sid) = val.get("session_id").and_then(|s| s.as_str()) {
                    session_id = Some(sid.to_string());
                }
            }
            Some("result") => {
                if let Some(r) = val.get("result").and_then(|r| r.as_str()) {
                    result_text = r.to_string();
                }
                // Extract usage from result message
                if let Some(u) = val.get("usage") {
                    usage = Some(TokenUsage {
                        input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()),
                        output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()),
                    });
                }
            }
            Some("assistant") => {
                // Extract tool_use entries from assistant message content
                if let Some(content) = val
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                                tool_usage.push(ToolUsageEntry {
                                    name: name.to_string(),
                                    input: block.get("input").cloned(),
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if !any_parsed {
        return parse_raw_output(raw);
    }

    // If no result line was found, collect any non-JSON text as the result
    if result_text.is_empty() {
        result_text = raw
            .lines()
            .filter(|l| serde_json::from_str::<serde_json::Value>(l.trim()).is_err())
            .collect::<Vec<_>>()
            .join("\n");
    }

    ParsedOutput {
        result: result_text,
        session_id,
        usage,
        tool_usage,
        is_structured: true,
    }
}

/// Parse Codex CLI NDJSON output.
///
/// Expected line types:
/// - `{"type":"thread.started","thread_id":"..."}` — thread metadata
/// - `{"type":"item.completed","item":{"type":"agent_message","text":"..."}}` — response
/// - `{"type":"turn.completed","usage":{"input_tokens":...,"output_tokens":...}}` — usage
fn parse_codex_output(raw: &str) -> ParsedOutput {
    let mut thread_id: Option<String> = None;
    let mut last_assistant_msg = String::new();
    let mut usage: Option<TokenUsage> = None;
    let mut any_parsed = false;

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        any_parsed = true;

        match val.get("type").and_then(|t| t.as_str()) {
            Some("thread.started") => {
                if let Some(tid) = val.get("thread_id").and_then(|s| s.as_str()) {
                    thread_id = Some(tid.to_string());
                }
            }
            Some("item.completed") => {
                // {"type":"item.completed","item":{"type":"agent_message","text":"..."}}
                if let Some(item) = val.get("item") {
                    if item.get("type").and_then(|t| t.as_str()) == Some("agent_message") {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            last_assistant_msg = text.to_string();
                        }
                    }
                }
            }
            Some("turn.completed") => {
                // {"type":"turn.completed","usage":{"input_tokens":...,"output_tokens":...}}
                if let Some(u) = val.get("usage") {
                    usage = Some(TokenUsage {
                        input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()),
                        output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()),
                    });
                }
            }
            _ => {}
        }
    }

    if !any_parsed {
        return parse_raw_output(raw);
    }

    ParsedOutput {
        result: last_assistant_msg,
        session_id: thread_id,
        usage,
        tool_usage: Vec::new(),
        is_structured: true,
    }
}

/// Parse Gemini CLI JSON output (single JSON object).
fn parse_gemini_output(raw: &str) -> ParsedOutput {
    let trimmed = raw.trim();

    // Try to parse as a single JSON object
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        // Gemini may return response text in various fields
        let result = val
            .get("response")
            .and_then(|r| r.as_str())
            .or_else(|| val.get("text").and_then(|t| t.as_str()))
            .or_else(|| val.get("result").and_then(|r| r.as_str()))
            .unwrap_or("")
            .to_string();

        return ParsedOutput {
            result,
            session_id: None, // Gemini uses index-based resume, not ID-based
            usage: None,
            tool_usage: Vec::new(),
            is_structured: true,
        };
    }

    // Fallback to raw text
    parse_raw_output(raw)
}

/// Fallback parser: wraps raw text with `is_structured: false`.
fn parse_raw_output(raw: &str) -> ParsedOutput {
    ParsedOutput {
        result: raw.to_string(),
        session_id: None,
        usage: None,
        tool_usage: Vec::new(),
        is_structured: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_output_full() {
        let raw = r#"{"type":"system","session_id":"sess-abc-123","tool_event":"init"}
{"type":"assistant","message":{"content":[{"type":"text","text":"Let me check..."},{"type":"tool_use","id":"tu_1","name":"Read","input":{"path":"/tmp/test.rs"}}]}}
{"type":"result","result":"The file contains a Rust function.","usage":{"input_tokens":150,"output_tokens":42}}"#;

        let parsed = parse_claude_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.session_id.as_deref(), Some("sess-abc-123"));
        assert_eq!(parsed.result, "The file contains a Rust function.");
        assert_eq!(parsed.usage.as_ref().unwrap().input_tokens, Some(150));
        assert_eq!(parsed.usage.as_ref().unwrap().output_tokens, Some(42));
        assert_eq!(parsed.tool_usage.len(), 1);
        assert_eq!(parsed.tool_usage[0].name, "Read");
    }

    #[test]
    fn test_parse_claude_output_no_result_line() {
        // If there's no result line, non-JSON text is collected
        let raw = r#"{"type":"system","session_id":"sess-xyz"}
This is plain text output
from the assistant"#;

        let parsed = parse_claude_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.session_id.as_deref(), Some("sess-xyz"));
        assert!(parsed.result.contains("This is plain text output"));
    }

    #[test]
    fn test_parse_claude_output_invalid_json_fallback() {
        let raw = "Just plain text, no JSON at all.";
        let parsed = parse_claude_output(raw);
        assert!(!parsed.is_structured);
        assert_eq!(parsed.result, raw);
    }

    #[test]
    fn test_parse_codex_output_full() {
        let raw = r#"{"type":"thread.started","thread_id":"thread-xyz-789"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"Hello, world!"}}
{"type":"turn.completed","usage":{"input_tokens":100,"cached_input_tokens":50,"output_tokens":25}}"#;

        let parsed = parse_codex_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.session_id.as_deref(), Some("thread-xyz-789"));
        assert_eq!(parsed.result, "Hello, world!");
        assert_eq!(parsed.usage.as_ref().unwrap().input_tokens, Some(100));
        assert_eq!(parsed.usage.as_ref().unwrap().output_tokens, Some(25));
    }

    #[test]
    fn test_parse_codex_output_turn_completed_usage() {
        let raw = r#"{"type":"thread.started","thread_id":"t-1"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"Done."}}
{"type":"turn.completed","usage":{"input_tokens":50,"output_tokens":10}}"#;

        let parsed = parse_codex_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.usage.as_ref().unwrap().input_tokens, Some(50));
        assert_eq!(parsed.usage.as_ref().unwrap().output_tokens, Some(10));
    }

    #[test]
    fn test_parse_codex_output_invalid_json_fallback() {
        let raw = "Raw codex output without JSON";
        let parsed = parse_codex_output(raw);
        assert!(!parsed.is_structured);
        assert_eq!(parsed.result, raw);
    }

    #[test]
    fn test_parse_gemini_output_json() {
        let raw = r#"{"response": "The answer is 42.", "model": "gemini-2.5-pro"}"#;
        let parsed = parse_gemini_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.result, "The answer is 42.");
        assert!(parsed.session_id.is_none());
    }

    #[test]
    fn test_parse_gemini_output_text_field() {
        let raw = r#"{"text": "Some text response"}"#;
        let parsed = parse_gemini_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.result, "Some text response");
    }

    #[test]
    fn test_parse_gemini_output_raw_fallback() {
        let raw = "Just plain gemini output";
        let parsed = parse_gemini_output(raw);
        assert!(!parsed.is_structured);
        assert_eq!(parsed.result, raw);
    }

    #[test]
    fn test_parse_raw_output() {
        let raw = "Hello, I am raw output from grok.";
        let parsed = parse_raw_output(raw);
        assert!(!parsed.is_structured);
        assert_eq!(parsed.result, raw);
        assert!(parsed.session_id.is_none());
        assert!(parsed.usage.is_none());
        assert!(parsed.tool_usage.is_empty());
    }

    #[test]
    fn test_parse_output_dispatch() {
        let raw = r#"{"type":"system","session_id":"s1"}
{"type":"result","result":"ok"}"#;

        let parsed = parse_output("claude", raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.session_id.as_deref(), Some("s1"));

        let parsed = parse_output("grok", "hello");
        assert!(!parsed.is_structured);
        assert_eq!(parsed.result, "hello");

        let parsed = parse_output("ollama", "hi");
        assert!(!parsed.is_structured);
    }

    #[test]
    fn test_parse_claude_empty_lines_skipped() {
        let raw = r#"
{"type":"system","session_id":"s2"}

{"type":"result","result":"done"}
"#;
        let parsed = parse_claude_output(raw);
        assert!(parsed.is_structured);
        assert_eq!(parsed.session_id.as_deref(), Some("s2"));
        assert_eq!(parsed.result, "done");
    }

    #[test]
    fn test_parse_codex_multiple_items() {
        // Should use the last agent_message item
        let raw = r#"{"type":"thread.started","thread_id":"t-1"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"First response"}}
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"Final response"}}"#;

        let parsed = parse_codex_output(raw);
        assert_eq!(parsed.result, "Final response");
    }

    #[test]
    fn test_parsed_output_serializes() {
        let parsed = ParsedOutput {
            result: "test".to_string(),
            session_id: Some("s1".to_string()),
            usage: Some(TokenUsage {
                input_tokens: Some(10),
                output_tokens: Some(5),
            }),
            tool_usage: vec![ToolUsageEntry {
                name: "Read".to_string(),
                input: None,
            }],
            is_structured: true,
        };
        let json = serde_json::to_string(&parsed).unwrap();
        assert!(json.contains("\"session_id\":\"s1\""));
        assert!(json.contains("\"input_tokens\":10"));
    }
}
