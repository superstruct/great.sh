import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { mcpBridgeOutput } from '@/data/commands'
import { motion } from 'motion/react'

const bridgeFeatures = [
  {
    label: '5 backends',
    desc: 'Gemini, Codex, Claude, Grok, and Ollama — any mix of cloud and local models.',
  },
  {
    label: '4 presets',
    desc: 'minimal (1 tool), agent (6), research (8), full (9) — pick the surface area you need.',
  },
  {
    label: 'Zero Node.js',
    desc: 'Pure Rust, compiled into the great binary. No npx, no node_modules, no npm audit.',
  },
  {
    label: 'Auto-registered',
    desc: 'great apply writes the bridge entry into .mcp.json so Claude Code discovers it automatically.',
  },
]

export function Bridge() {
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
                — five backends, zero Node.js
              </span>
            </h2>
            <p className="text-text-secondary mb-6 leading-relaxed">
              A single MCP server that multiplexes five AI CLI backends over
              JSON-RPC 2.0 stdio. No sidecar processes, no JavaScript runtime
              — just one Rust binary speaking the Model Context Protocol.
            </p>
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
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
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
