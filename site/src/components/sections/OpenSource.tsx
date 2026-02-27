import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { Github } from 'lucide-react'
import { motion } from 'motion/react'

export function OpenSource() {
  return (
    <AnimatedSection id="open-source">
      <Container>
        <div className="text-center max-w-2xl mx-auto">
          <h2 className="font-display text-3xl md:text-4xl text-text-primary mb-4">
            Open source. Free forever.
          </h2>
          <p className="text-text-secondary mb-6 leading-relaxed">
            The CLI is free and open source under the Apache 2.0 license. Every feature works
            without an account. No paywalls, no telemetry &mdash; the tool is yours to keep, forever.
          </p>
          <p className="text-text-tertiary text-sm mb-10">
            BYO credentials. We never see your API keys. Secrets stay in your system keychain.
          </p>

          <motion.div
            initial={{ opacity: 0, y: 10 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.4, delay: 0.2 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-4"
          >
            <a
              href="https://github.com/superstruct/great.sh"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 bg-text-primary text-bg-primary font-medium px-8 py-3 rounded-lg hover:bg-text-secondary transition-colors"
            >
              <Github size={18} />
              View on GitHub
            </a>
            <a
              href="https://github.com/superstruct/great.sh/discussions"
              target="_blank"
              rel="noopener noreferrer"
              className="text-text-secondary hover:text-accent transition-colors"
            >
              Join the discussion &rarr;
            </a>
          </motion.div>

          <p className="text-text-tertiary text-sm mt-6">
            great.sh is in alpha. Found a bug or have a suggestion?{' '}
            <a
              href="https://github.com/superstruct/great.sh/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="text-accent hover:text-accent-hover transition-colors"
            >
              Open an issue
            </a>
            {' '}or{' '}
            <a
              href="https://github.com/superstruct/great.sh/discussions"
              target="_blank"
              rel="noopener noreferrer"
              className="text-accent hover:text-accent-hover transition-colors"
            >
              start a discussion
            </a>.
          </p>
        </div>
      </Container>
    </AnimatedSection>
  )
}
