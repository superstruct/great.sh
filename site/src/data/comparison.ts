export interface ComparisonRow {
  feature: string
  great: boolean | string
  chezmoi: boolean | string
  mise: boolean | string
  nix: boolean | string
  mcpm: boolean | string
  manual: boolean | string
}

export const comparisonData: ComparisonRow[] = [
  {
    feature: 'One-command full setup',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Declarative config file',
    great: 'great.toml',
    chezmoi: 'chezmoi.toml',
    mise: 'mise.toml',
    nix: 'flake.nix',
    mcpm: false,
    manual: false,
  },
  {
    feature: 'AI CLI tool installation',
    great: true,
    chezmoi: false,
    mise: false,
    nix: 'Partial',
    mcpm: false,
    manual: true,
  },
  {
    feature: 'AI agent orchestration loop',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'MCP server management',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: 'List only',
    manual: true,
  },
  {
    feature: 'Built-in multi-AI bridge (no Node.js)',
    great: true,
    chezmoi: false,
    mise: false,
    nix: false,
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Credential management',
    great: true,
    chezmoi: 'Partial',
    mise: false,
    nix: false,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Cross-machine sync',
    great: 'Local only',
    chezmoi: 'Git-based',
    mise: false,
    nix: 'Git-based',
    mcpm: false,
    manual: false,
  },
  {
    feature: 'Runtime version management',
    great: 'Via mise',
    chezmoi: false,
    mise: true,
    nix: true,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Dotfiles management',
    great: true,
    chezmoi: true,
    mise: false,
    nix: true,
    mcpm: false,
    manual: true,
  },
  {
    feature: 'Learning curve',
    great: 'Minutes',
    chezmoi: 'Hours',
    mise: 'Hours',
    nix: 'Weeks',
    mcpm: 'Minutes',
    manual: 'Days',
  },
]
