import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { templates } from '@/data/templates'
import { motion } from 'motion/react'

export function Templates() {
  return (
    <AnimatedSection id="templates">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          Start with a template
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
          Curated environment configs encoding best-practice AI dev setups. Use as-is or customize.
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
              <div className="mb-4">
                <code className="text-accent text-xs bg-accent-muted px-2 py-1 rounded font-mono">
                  {template.id}
                </code>
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
      </Container>
    </AnimatedSection>
  )
}
