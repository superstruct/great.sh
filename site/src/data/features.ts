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
      'The great.sh Loop: 16 specialized AI roles installed into Claude Code with one command. Requirements, specs, builds, tests, security audits, performance checks, code reviews, UX inspections, visual reviews, docs, and deploys — orchestrated as a team. Run great loop install to set it up.',
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
