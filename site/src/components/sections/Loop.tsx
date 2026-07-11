import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { loopInstallOutput } from '@/data/commands'
import { motion } from 'motion/react'

const phases = [
  {
    label: 'Phase 1 — Plan',
    agents: [
      { name: 'Lead', role: 'Spec + self-review', methodology: 'Your session picks the task, writes the spec, and hunts its own gaps before any code' },
      { name: 'Scout', role: 'Recon (optional)', methodology: 'Read-only map of large or unfamiliar change surfaces' },
    ],
    flow: 'sequential' as const,
  },
  {
    label: 'Phase 2 — Build & Verify',
    agents: [
      { name: 'Builder', role: 'Build', methodology: 'Turns specs into working code, runs all quality gates, answers findings with evidence' },
      { name: 'Verifier', role: 'Adversarial verify', methodology: 'Tries to prove the change broken or insecure — correctness, regression, security, performance. Findings must cite reproductions' },
      { name: 'Reviewer', role: 'Quality review', methodology: 'Read-only review — structure, simplification, UX, output design, docs' },
    ],
    flow: 'parallel' as const,
    note: 'Ends when gates are green and no CONFIRMED findings remain — not after a fixed number of rounds',
  },
  {
    label: 'Phase 3 — Finish',
    agents: [
      { name: 'Lead', role: 'Commit + observe', methodology: 'Commits only with all gates green, writes the observer report, one config change max' },
    ],
    flow: 'sequential' as const,
  },
]

const slashCommands = [
  { cmd: '/great:backlog', desc: 'Capture requirements into .tasks/backlog/ — run this first' },
  { cmd: '/great:loop', desc: 'Full evidence-gated development cycle' },
  { cmd: '/great:bugfix', desc: 'Targeted fix: reproduce, patch, verify, commit' },
  { cmd: '/great:deploy', desc: 'Build and verify release artifacts with a rollback path defined up front' },
  { cmd: '/great:discover', desc: 'UX discovery sweep — reviewer maps journeys, lead files issues' },
]

export function Loop() {
  return (
    <AnimatedSection id="loop">
      <Container>
        {/* Heading */}
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          great loop{' '}
          <span className="text-text-secondary">— evidence over role-play</span>
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto leading-relaxed">
          The great.sh Loop is an evidence-gated agent methodology that ships
          inside every great.sh install. One command configures Claude Code with
          a lean team — a builder, an adversarial verifier, and a quality
          reviewer, plus an optional scout — where no agent declares success
          without cited command output and phases end when exit criteria are
          met, not after a fixed number of rounds. Type{' '}
          <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
            /great:backlog
          </code>{' '}
          to capture requirements, then{' '}
          <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded font-mono">
            /great:loop
          </code>{' '}
          to execute.
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
              <div className={
                phase.agents.length > 5
                  ? 'grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2'
                  : 'flex flex-wrap items-center gap-2'
              }>
                {phase.agents.map((agent, i) => (
                  <div key={agent.name} className={
                    phase.agents.length > 5 ? '' : 'flex items-center gap-2'
                  }>
                    <div className="bg-bg-secondary border border-border rounded-lg px-4 py-2.5 hover:border-accent/30 transition-colors">
                      <div className="font-display text-sm text-text-primary">
                        {agent.name}
                      </div>
                      <div className="text-text-tertiary text-xs">
                        {agent.role}
                      </div>
                      <div className="text-text-tertiary text-xs mt-0.5 max-w-[220px]">
                        {agent.methodology}
                      </div>
                    </div>
                    {phase.agents.length <= 5 && i < phase.agents.length - 1 && (
                      <span className="hidden xl:inline text-text-tertiary text-sm font-mono">
                        {phase.flow === 'parallel' ? '+' : '→'}
                      </span>
                    )}
                  </div>
                ))}
              </div>
              {'note' in phase && phase.note && (
                <div className="text-text-tertiary text-xs font-mono mt-2 pl-1">
                  + {phase.note}
                </div>
              )}
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
              Five slash commands
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
