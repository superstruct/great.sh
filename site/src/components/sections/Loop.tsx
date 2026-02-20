import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { loopInstallOutput } from '@/data/commands'
import { motion } from 'motion/react'

const phases = [
  {
    label: 'Phase 1 -- Sequential',
    agents: [
      { name: 'Nightingale', role: 'Requirements' },
      { name: 'Lovelace', role: 'Spec' },
      { name: 'Socrates', role: 'Review' },
      { name: 'Humboldt', role: 'Scout' },
    ],
    flow: 'sequential' as const,
  },
  {
    label: 'Phase 2 -- Parallel Team',
    agents: [
      { name: 'Da Vinci', role: 'Build' },
      { name: 'Von Braun', role: 'Deploy' },
      { name: 'Turing', role: 'Test' },
      { name: 'Kerckhoffs', role: 'Security' },
      { name: 'Nielsen', role: 'UX' },
    ],
    flow: 'parallel' as const,
  },
  {
    label: 'Phase 3 -- Finish',
    agents: [
      { name: 'Rams', role: 'Visual QA' },
      { name: 'Hopper', role: 'Commit' },
      { name: 'Knuth', role: 'Docs' },
      { name: 'Gutenberg', role: 'Doc Commit' },
      { name: 'Deming', role: 'Observe' },
    ],
    flow: 'sequential' as const,
  },
]

const slashCommands = [
  { cmd: '/loop', desc: 'Full development cycle' },
  { cmd: '/bugfix', desc: 'Diagnose and fix a bug' },
  { cmd: '/deploy', desc: 'Build, test, and ship' },
  { cmd: '/discover', desc: 'Explore and document a codebase' },
]

export function Loop() {
  return (
    <AnimatedSection id="loop">
      <Container>
        {/* Heading */}
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          great loop{' '}
          <span className="text-text-secondary">— 13 agents, one command</span>
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto leading-relaxed">
          The great.sh Loop is a 13-role AI agent orchestration methodology that
          ships inside every great.sh install. One command configures Claude Code
          with a full team: requirements analysts, spec writers, builders, testers,
          security auditors, UX reviewers, documenters, and an observer.
          Type{' '}
          <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
            /loop [task]
          </code>{' '}
          and the team goes to work.
        </p>

        {/* Agent flow */}
        <div className="space-y-8 mb-16">
          {phases.map((phase, phaseIdx) => (
            <motion.div
              key={phase.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.4, delay: phaseIdx * 0.15 }}
            >
              <div className="text-text-tertiary text-xs font-mono uppercase tracking-wider mb-3">
                {phase.label}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {phase.agents.map((agent, i) => (
                  <div key={agent.name} className="flex items-center gap-2">
                    <div className="bg-bg-secondary border border-border rounded-lg px-4 py-2.5 hover:border-accent/30 transition-colors">
                      <div className="font-display text-sm text-text-primary">
                        {agent.name}
                      </div>
                      <div className="text-text-tertiary text-xs">
                        {agent.role}
                      </div>
                    </div>
                    {i < phase.agents.length - 1 && (
                      <span className="text-text-tertiary text-sm font-mono">
                        {phase.flow === 'parallel' ? '+' : '\u2192'}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </motion.div>
          ))}
        </div>

        {/* Two-column: terminal + slash commands */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.1 }}
          >
            <TerminalWindow title="great loop install --project">
              <pre className="text-xs leading-relaxed text-text-secondary whitespace-pre-wrap">
                {loopInstallOutput}
              </pre>
            </TerminalWindow>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: '-50px' }}
            transition={{ duration: 0.5, delay: 0.2 }}
            className="flex flex-col justify-center"
          >
            <h3 className="font-display text-xl text-text-primary mb-6">
              Four slash commands
            </h3>
            <div className="space-y-4">
              {slashCommands.map((sc) => (
                <div key={sc.cmd} className="flex items-baseline gap-3">
                  <code className="text-accent text-sm bg-accent-muted px-2 py-1 rounded font-mono flex-shrink-0">
                    {sc.cmd}
                  </code>
                  <span className="text-text-secondary text-sm">{sc.desc}</span>
                </div>
              ))}
            </div>

            <div className="mt-8 space-y-2">
              <code className="block text-text-secondary text-xs font-mono">
                <span className="text-accent">$</span> great loop install{' '}
                <span className="text-text-tertiary"># global: adds agents to ~/.claude/</span>
              </code>
              <code className="block text-text-secondary text-xs font-mono">
                <span className="text-accent">$</span> great loop install --project{' '}
                <span className="text-text-tertiary"># also sets up .tasks/ kanban</span>
              </code>
            </div>
          </motion.div>
        </div>
      </Container>
    </AnimatedSection>
  )
}
