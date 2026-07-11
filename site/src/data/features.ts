export interface Feature {
  title: string
  description: string
  icon: string
}

export const features: Feature[] = [
  {
    title: 'One Command Setup',
    description:
      'From a blank machine to a fully configured AI dev environment. Install tools, runtimes, and shell config in a single command.',
    icon: 'terminal',
  },
  {
    title: 'AI Agent Orchestration',
    description:
      'The great.sh Loop: an evidence-gated agent team installed into Claude Code with one command. A builder implements, an adversarial verifier tries to prove the change broken or insecure, a reviewer guards quality — and nothing ships without cited command output. Run great loop install to set it up.',
    icon: 'brain',
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
      'Store API keys in your system keychain, import from .env files, and snapshot config locally. BYO credentials \u2014 cloud sync coming soon.',
    icon: 'shield',
  },
  {
    title: 'Built-in AI Bridge',
    description:
      'Use Claude Code to call Gemini, Codex, Grok, or Ollama — all from one bridge that ships inside the great binary. No Node.js dependency, no extra install.',
    icon: 'bridge',
  },
]
