import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { features } from '@/data/features'
import { Terminal, BrainCircuit, Server, ShieldCheck, Cable } from 'lucide-react'
import { motion } from 'motion/react'

const iconMap = {
  terminal: Terminal,
  brain: BrainCircuit,
  server: Server,
  shield: ShieldCheck,
  bridge: Cable,
}

export function Features() {
  return (
    <AnimatedSection id="features">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          Everything you need. Nothing you don't.
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-2xl mx-auto">
          great.sh is the only tool that touches all five layers: template provisioning,
          AI agent orchestration, MCP configuration, credential management, and cross-machine sync.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {features.map((feature, i) => {
            const Icon = iconMap[feature.icon as keyof typeof iconMap]
            return (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true, margin: '-50px' }}
                transition={{ duration: 0.4, delay: i * 0.1 }}
                className={`bg-bg-secondary border border-border rounded-xl p-8 hover:border-accent/30 transition-colors${
                  i === features.length - 1 && features.length % 2 !== 0 ? ' md:col-span-2 md:max-w-lg md:mx-auto' : ''
                }`}
              >
                <div className="w-10 h-10 rounded-lg bg-accent-muted flex items-center justify-center mb-4">
                  <Icon size={20} className="text-accent" />
                </div>
                <h3 className="font-display text-xl text-text-primary mb-2">{feature.title}</h3>
                <p className="text-text-secondary text-sm leading-relaxed">{feature.description}</p>
              </motion.div>
            )
          })}
        </div>
      </Container>
    </AnimatedSection>
  )
}
