import { useState } from 'react'
import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { bridgeAddCommand, bridgeDemoOutput, mcpBridgeOutput } from '@/data/commands'
import { motion } from 'motion/react'
import { Check, Copy } from 'lucide-react'

const bridgeFeatures = [
  {
    label: '5 backends',
    desc: 'Gemini, Codex, Claude, Grok, and Ollama — any mix of cloud and local models.',
  },
  {
    label: 'Sync + async',
    desc: 'One-shot prompts, or background tasks with wait, get_result, timeouts, and auto-cleanup.',
  },
  {
    label: '4 presets',
    desc: 'minimal (1 tool), agent (7), research (9), full (10) — pick the surface area you need.',
  },
  {
    label: 'Zero Node.js',
    desc: 'Pure Rust, compiled into the great binary. No npx, no node_modules, no npm audit.',
  },
  {
    label: 'Guardrails',
    desc: 'allowed-dirs allowlist for file-reading tools, per-task timeouts, opt-out of auto-approval flags.',
  },
]

export function Bridge() {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(bridgeAddCommand)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <AnimatedSection id="bridge">
      <Container>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-start">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="flex flex-col justify-center"
          >
            <h2 className="font-display text-3xl md:text-4xl text-text-primary mb-4">
              great mcp-bridge{' '}
              <span className="text-text-secondary">
                — one MCP server, five AI backends
              </span>
            </h2>
            <p className="text-text-secondary mb-6 leading-relaxed">
              A single MCP server that multiplexes five AI CLI backends over
              JSON-RPC 2.0 stdio. No sidecar processes, no JavaScript runtime
              — just one Rust binary speaking the Model Context Protocol.
              Register it once and Claude Code can consult Gemini, Codex,
              Grok, or a local Ollama model mid-conversation.
            </p>

            <div className="mb-6">
              <TerminalWindow title="~">
                <div className="flex items-center justify-between gap-4">
                  <div className="text-xs md:text-sm overflow-x-auto">
                    <span className="text-accent">$</span>{' '}
                    <span className="text-text-primary">{bridgeAddCommand}</span>
                  </div>
                  <button
                    onClick={handleCopy}
                    className="flex-shrink-0 p-1.5 rounded hover:bg-bg-tertiary text-text-tertiary hover:text-text-primary transition-all"
                    aria-label="Copy bridge registration command"
                  >
                    {copied ? <Check size={14} className="text-accent" /> : <Copy size={14} />}
                  </button>
                </div>
              </TerminalWindow>
            </div>

            <ul className="space-y-3 text-text-secondary text-sm">
              {bridgeFeatures.map((f) => (
                <li key={f.label} className="flex items-start gap-2">
                  <span className="text-accent mt-0.5">&#10003;</span>
                  <span>
                    <strong className="text-text-primary">{f.label}</strong>
                    {' — '}
                    {f.desc}
                  </span>
                </li>
              ))}
            </ul>

            <p className="text-text-tertiary text-sm mt-6">
              Full reference — backends, config schema, security notes:{' '}
              <a
                href="https://github.com/superstruct/great.sh/blob/main/docs/mcp-bridge.md"
                target="_blank"
                rel="noopener noreferrer"
                className="text-accent hover:text-accent-hover transition-colors"
              >
                docs/mcp-bridge.md
              </a>
            </p>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="space-y-6"
          >
            <TerminalWindow title="claude — via great-bridge">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {bridgeDemoOutput}
              </pre>
            </TerminalWindow>
            <TerminalWindow title="great mcp-bridge --preset agent">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {mcpBridgeOutput}
              </pre>
            </TerminalWindow>
          </motion.div>
        </div>
      </Container>
    </AnimatedSection>
  )
}
