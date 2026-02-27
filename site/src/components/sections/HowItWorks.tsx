import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { initWizardOutput } from '@/data/commands'
import { motion } from 'motion/react'

const steps = [
  {
    number: '01',
    title: 'Install',
    description: 'One curl command downloads the great CLI binary. macOS, Ubuntu, WSL2.',
    command: 'curl -sS https://great.sh/install.sh | sh',
  },
  {
    number: '02',
    title: 'Initialize',
    description: 'Interactive wizard picks your AI agents, MCP servers, and tools. Enter API keys once.',
    command: 'great init',
  },
  {
    number: '03',
    title: 'Code',
    description: 'Claude Code launches with all MCP servers connected. Codex and Gemini available as tools.',
    command: 'claude',
  },
  {
    number: '04',
    title: 'Snapshot',
    description: 'Save a local config snapshot. Restore it anytime, or on a fresh install.',
    command: 'great sync push',
  },
  {
    number: '05',
    title: 'Start the Loop',
    description: 'Install the 16-role agent team into Claude Code. Type /backlog to capture requirements, then /loop to build.',
    command: 'great loop install --project',
  },
]

export function HowItWorks() {
  return (
    <AnimatedSection id="how-it-works">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          Zero to hero in five steps
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
          From blank machine to fully configured AI dev environment.
        </p>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
          <div className="space-y-8">
            {steps.map((step, i) => (
              <motion.div
                key={step.number}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.4, delay: i * 0.1 }}
                className="flex gap-4"
              >
                <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-accent-muted flex items-center justify-center">
                  <span className="text-accent font-mono text-sm font-bold">{step.number}</span>
                </div>
                <div>
                  <h3 className="font-display text-lg text-text-primary mb-1">{step.title}</h3>
                  <p className="text-text-secondary text-sm mb-2">{step.description}</p>
                  <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
                    $ {step.command}
                  </code>
                </div>
              </motion.div>
            ))}
          </div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <TerminalWindow title="great init">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {initWizardOutput}
              </pre>
            </TerminalWindow>
          </motion.div>
        </div>
      </Container>
    </AnimatedSection>
  )
}
