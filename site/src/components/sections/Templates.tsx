import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { templates } from '@/data/templates'
import { motion } from 'motion/react'

export function Templates() {
  return (
    <AnimatedSection id="templates">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          Template marketplace
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
          Premium environment configs available on{' '}
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent hover:underline"
          >
            architecton.ai
          </a>
          . Install with <code className="text-accent font-mono text-sm">great template apply &lt;id&gt;</code>.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {templates.map((template, i) => (
            <motion.div
              key={template.id}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-50px' }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="bg-bg-secondary border border-border rounded-xl p-6 hover:border-accent/30 transition-colors flex flex-col"
            >
              <div className="mb-4 flex items-center gap-2">
                <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
                  {template.id}
                </code>
                {template.source === 'architecton.ai' && (
                  <span className="text-xs text-text-tertiary border border-border px-2 py-0.5 rounded">
                    via architecton.ai
                  </span>
                )}
              </div>
              <h3 className="font-display text-lg text-text-primary mb-2">{template.name}</h3>
              <p className="text-text-secondary text-sm mb-4 flex-1">{template.description}</p>

              <div className="space-y-3 pt-4 border-t border-border">
                <div>
                  <span className="text-text-tertiary text-xs uppercase tracking-wider">Agents</span>
                  <div className="flex flex-wrap gap-1.5 mt-1">
                    {template.agents.map((a) => (
                      <span key={a} className="text-xs bg-bg-tertiary text-text-secondary px-2 py-0.5 rounded">
                        {a}
                      </span>
                    ))}
                  </div>
                </div>
                <div>
                  <span className="text-text-tertiary text-xs uppercase tracking-wider">MCP Servers</span>
                  <div className="flex flex-wrap gap-1.5 mt-1">
                    {template.mcpServers.map((s) => (
                      <span key={s} className="text-xs bg-bg-tertiary text-text-secondary px-2 py-0.5 rounded">
                        {s}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <div className="mt-12 text-center">
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-2 text-accent hover:underline font-medium"
          >
            Browse all templates on architecton.ai
            <span aria-hidden="true">&rarr;</span>
          </a>
        </div>
      </Container>
    </AnimatedSection>
  )
}
