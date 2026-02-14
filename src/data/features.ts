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
      'Claude Code as orchestrator with Codex, Gemini, and other AI agents configured as MCP servers. Multi-agent development out of the box.',
    icon: 'brain',
  },
  {
    title: 'MCP Server Management',
    description:
      'Install, configure, and credential-inject MCP servers from the official registry. Health checks, cross-client config sync, curated bundles.',
    icon: 'server',
  },
  {
    title: 'Cloud-Synced Credentials',
    description:
      'Zero-knowledge encrypted vault syncs API keys and config across machines. BYO credentials \u2014 we never see your keys.',
    icon: 'shield',
  },
]
