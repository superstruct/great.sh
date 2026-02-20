import { Container } from './Container'
import { Github } from 'lucide-react'

export function Footer() {
  return (
    <footer className="border-t border-border py-12">
      <Container>
        <div className="flex flex-col md:flex-row justify-between items-center gap-6">
          <div className="flex items-center gap-2 font-display text-lg text-text-primary">
            <span className="text-accent font-mono font-bold">&gt;</span>
            great.sh
          </div>
          <div className="flex items-center gap-6 text-text-secondary text-sm">
            <a
              href="https://github.com/great-sh/great"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-text-primary transition-colors flex items-center gap-1.5"
            >
              <Github size={16} /> GitHub
            </a>
            <span className="text-border">|</span>
            <span className="text-text-tertiary">
              Open source &middot; Apache-2.0 License
            </span>
          </div>
        </div>
        <div className="mt-6 text-center text-text-tertiary text-xs">
          Built by{' '}
          <a
            href="https://superstruct.nz"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-text-secondary transition-colors"
          >
            Superstruct
          </a>
          {' '}&middot; Part of the{' '}
          <a
            href="https://architecton.ai"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-text-secondary transition-colors"
          >
            architecton.ai
          </a>
          {' '}ecosystem
        </div>
      </Container>
    </footer>
  )
}
