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

  ? Enter your API keys (stored in encrypted vault):
    ANTHROPIC_API_KEY: ********************
    OPENAI_API_KEY: ********************

  [check] Installing tools: node 22, python 3.12, gh, ripgrep...
  [check] Configuring Claude Code with 2 AI agent MCP servers
  [check] Configuring 4 additional MCP servers
  [check] Injecting credentials into MCP server configs

  Your AI dev environment is ready!

  Run \`claude\` to start Claude Code with all MCP servers.
  Run \`great loop install\` to add the 16-role agent team.`

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

[secrets]
provider = "great-vault"
required = [
  "ANTHROPIC_API_KEY",
  "OPENAI_API_KEY",
  "GITHUB_TOKEN",
]`

export const loopInstallOutput = `$ great loop install --project

  great.sh Loop -- Installing agent team

  [check] 15 agent personas -> ~/.claude/agents/
  [check] 5 commands -> ~/.claude/commands/
  [check] Agent Teams config -> ~/.claude/teams/loop/
  [check] Settings with Agent Teams enabled -> ~/.claude/settings.json
  [check] .tasks/ created, .gitignore updated

  great.sh Loop installed!

  16 roles: 4 teammates + 11 subagents + 1 team lead
  Usage: claude -> /loop [task description]`
