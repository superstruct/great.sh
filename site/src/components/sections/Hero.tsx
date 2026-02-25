import { useState } from 'react'
import { motion } from 'motion/react'
import { Container } from '@/components/layout/Container'
import { TerminalWindow } from '@/components/shared/TerminalWindow'
import { installCommand } from '@/data/commands'
import { Check, Copy } from 'lucide-react'

export function Hero() {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(installCommand)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <section id="hero" className="min-h-screen flex items-center pt-16">
      <Container>
        <div className="flex flex-col items-center text-center max-w-4xl mx-auto">
          <motion.span
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className="text-xs font-mono text-accent border border-accent/30 bg-accent-muted px-3 py-1 rounded-full mb-4 inline-block"
          >
            alpha — open to testing &amp; feedback
          </motion.span>

          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="mb-8"
          >
            <div className="flex items-center gap-3">
              <span className="text-accent font-mono text-6xl md:text-7xl font-bold">&gt;</span>
              <h1 className="font-display text-6xl md:text-7xl lg:text-8xl text-text-primary">
                great<span className="text-text-secondary">.sh</span>
              </h1>
            </div>
          </motion.div>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.3 }}
            className="text-text-secondary text-lg md:text-xl mb-4 max-w-2xl"
          >
            The managed AI dev environment.
          </motion.p>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.4 }}
            className="text-text-tertiary text-base md:text-lg mb-12 max-w-2xl"
          >
            One command. 15 AI agents. Fully configured. Open source.
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.6 }}
            className="w-full max-w-xl mb-10"
          >
            <TerminalWindow title="~">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <span className="text-accent">$</span>{' '}
                  <span className="text-text-primary">{installCommand}</span>
                </div>
                <button
                  onClick={handleCopy}
                  className="flex-shrink-0 p-1.5 rounded hover:bg-bg-tertiary text-text-tertiary hover:text-text-primary transition-all"
                  aria-label="Copy install command"
                >
                  {copied ? <Check size={14} className="text-accent" /> : <Copy size={14} />}
                </button>
              </div>
            </TerminalWindow>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.8 }}
            className="flex flex-col sm:flex-row items-center gap-4"
          >
            <a
              href="#how-it-works"
              className="bg-accent text-bg-primary font-medium px-8 py-3 rounded-lg hover:bg-accent-hover transition-colors"
            >
              Get Started
            </a>
            <a
              href="#features"
              className="text-text-secondary hover:text-accent transition-colors"
            >
              Learn more &darr;
            </a>
          </motion.div>
        </div>
      </Container>
    </section>
  )
}
