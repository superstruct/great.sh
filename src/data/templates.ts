export interface Template {
  name: string
  id: string
  description: string
  agents: string[]
  mcpServers: string[]
  tools: string[]
}

export const templates: Template[] = [
  {
    name: 'AI Full Stack (TypeScript)',
    id: 'ai-fullstack-ts',
    description:
      'TypeScript full-stack development with Claude Code + Codex + Gemini, GitHub/Filesystem/Memory/Playwright MCP servers.',
    agents: ['Claude Code', 'Codex CLI', 'Gemini CLI'],
    mcpServers: ['GitHub', 'Filesystem', 'Memory', 'Playwright', 'Brave Search'],
    tools: ['Node 22', 'TypeScript', 'gh', 'Docker', 'Starship', 'fzf', 'ripgrep'],
  },
  {
    name: 'AI Full Stack (Python)',
    id: 'ai-fullstack-py',
    description:
      'Python full-stack development with uv package manager, PostgreSQL MCP server, and full AI agent setup.',
    agents: ['Claude Code', 'Codex CLI', 'Gemini CLI'],
    mcpServers: ['GitHub', 'Filesystem', 'Memory', 'PostgreSQL', 'Playwright'],
    tools: ['Python 3.12', 'uv', 'gh', 'Docker', 'Starship', 'fzf', 'ripgrep'],
  },
  {
    name: 'AI Data Science',
    id: 'ai-data-science',
    description:
      'Python data science with CUDA support, Jupyter, large-context Gemini for data analysis, and database MCP servers.',
    agents: ['Claude Code', 'Gemini CLI'],
    mcpServers: ['Filesystem', 'Memory', 'PostgreSQL', 'Brave Search'],
    tools: ['Python 3.12', 'Jupyter', 'CUDA', 'uv', 'Starship'],
  },
  {
    name: 'AI DevOps',
    id: 'ai-devops',
    description:
      'Infrastructure-as-code with AWS CLI, Terraform, Docker, Kubernetes MCP servers, Claude Code + Codex.',
    agents: ['Claude Code', 'Codex CLI'],
    mcpServers: ['AWS', 'Docker', 'Kubernetes', 'GitHub', 'Filesystem'],
    tools: ['Terraform', 'AWS CLI', 'Docker', 'kubectl', 'gh', 'Starship'],
  },
  {
    name: 'AI Minimal',
    id: 'ai-minimal',
    description:
      'Lightest footprint. Just Claude Code with Filesystem and Memory MCP servers. Perfect starting point.',
    agents: ['Claude Code'],
    mcpServers: ['Filesystem', 'Memory'],
    tools: ['gh', 'Starship', 'fzf', 'ripgrep'],
  },
]
