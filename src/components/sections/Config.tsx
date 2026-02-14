import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { CodeBlock } from '@/components/shared/CodeBlock'
import { sampleToml } from '@/data/commands'

export function Config() {
  return (
    <AnimatedSection id="config">
      <Container>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-start">
          <div>
            <h2 className="font-display text-3xl md:text-4xl text-text-primary mb-4">
              One file. Complete environment.
            </h2>
            <p className="text-text-secondary mb-6 leading-relaxed">
              <code className="text-accent text-sm bg-accent-muted px-1.5 py-0.5 rounded">great.toml</code>{' '}
              declares your entire AI dev environment &mdash; tools, runtimes, AI agents, MCP servers,
              and credential references. Commit it to version control. Share it with your team.
              Apply it on any machine.
            </p>
            <ul className="space-y-3 text-text-secondary text-sm">
              <li className="flex items-start gap-2">
                <span className="text-accent mt-0.5">&#10003;</span>
                Declarative &mdash; like docker-compose for your dev environment
              </li>
              <li className="flex items-start gap-2">
                <span className="text-accent mt-0.5">&#10003;</span>
                TOML format &mdash; no YAML ambiguity, supports comments
              </li>
              <li className="flex items-start gap-2">
                <span className="text-accent mt-0.5">&#10003;</span>
                Secrets referenced, never embedded &mdash; safe to commit
              </li>
              <li className="flex items-start gap-2">
                <span className="text-accent mt-0.5">&#10003;</span>
                Platform overlays &mdash; macOS, Linux, WSL2 specific config
              </li>
              <li className="flex items-start gap-2">
                <span className="text-accent mt-0.5">&#10003;</span>
                Composable templates &mdash; merge multiple configs
              </li>
            </ul>
          </div>
          <CodeBlock
            code={sampleToml}
            language="toml"
            filename="great.toml"
          />
        </div>
      </Container>
    </AnimatedSection>
  )
}
