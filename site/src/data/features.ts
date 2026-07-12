export interface Feature {
  title: string
  description: string
  icon: string
}

export const features: Feature[] = [
  {
    title: 'One MCP Server, Five Backends',
    description:
      'The great.sh MCP bridge multiplexes Gemini, Codex, Claude, Grok, and local Ollama behind one stdio MCP server — sync prompts, async tasks, research and code-analysis tools. Pure Rust, no Node.js. One claude mcp add and every backend is on tap.',
    icon: 'bridge',
  },
  {
    title: 'One Command Setup',
    description:
      'From a blank machine to a fully configured AI dev environment. Install tools, runtimes, and shell config in a single command.',
    icon: 'terminal',
  },
  {
    title: 'MCP Server Management',
    description:
      'Install, configure, and credential-inject external MCP servers from the official registry. Health checks, cross-client config sync, and curated bundles for third-party tools.',
    icon: 'server',
  },
  {
    title: 'Credential Vault',
    description:
      'Source API keys from env, 1Password, Bitwarden, or your system keychain and inject them where tools expect them. BYO credentials — nothing leaves your machine.',
    icon: 'shield',
  },
  {
    title: 'AI Agent Orchestration',
    description:
      'The great.sh Loop: an evidence-gated agent team installed into Claude Code with one command. A builder implements, an adversarial verifier tries to prove the change broken or insecure, a reviewer guards quality — and nothing ships without cited command output. Run great loop install to set it up.',
    icon: 'brain',
  },
]
