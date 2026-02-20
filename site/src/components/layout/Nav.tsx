import { useState, useEffect } from 'react'
import { cn } from '@/lib/utils'
import { Menu, X, Github } from 'lucide-react'

const navLinks = [
  { label: 'Features', href: '#features' },
  { label: 'Config', href: '#config' },
  { label: 'How it Works', href: '#how-it-works' },
  { label: 'Loop', href: '#loop' },
  { label: 'Templates', href: '#templates' },
  { label: 'Compare', href: '#compare' },
]

export function Nav() {
  const [scrolled, setScrolled] = useState(false)
  const [mobileOpen, setMobileOpen] = useState(false)

  useEffect(() => {
    const handleScroll = () => setScrolled(window.scrollY > 50)
    window.addEventListener('scroll', handleScroll)
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  return (
    <nav
      className={cn(
        'fixed top-0 left-0 right-0 z-50 transition-all duration-300',
        scrolled
          ? 'bg-bg-primary/80 backdrop-blur-xl border-b border-border'
          : 'bg-transparent'
      )}
    >
      <div className="mx-auto max-w-site px-6 md:px-12 flex items-center justify-between h-16">
        <a href="#" className="flex items-center gap-2 font-display text-xl text-text-primary">
          <span className="text-accent font-mono font-bold">&gt;</span>
          great.sh
          <span className="text-[10px] font-mono text-accent border border-accent/30 bg-accent-muted px-1.5 py-0.5 rounded-full">
            beta
          </span>
        </a>

        <div className="hidden md:flex items-center gap-8">
          {navLinks.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="text-sm text-text-secondary hover:text-accent transition-colors"
            >
              {link.label}
            </a>
          ))}
          <a
            href="https://github.com/superstruct/great.sh"
            target="_blank"
            rel="noopener noreferrer"
            className="text-text-secondary hover:text-text-primary transition-colors"
          >
            <Github size={20} />
          </a>
        </div>

        <button
          className="md:hidden text-text-primary"
          onClick={() => setMobileOpen(!mobileOpen)}
          aria-label={mobileOpen ? 'Close menu' : 'Open menu'}
        >
          {mobileOpen ? <X size={24} /> : <Menu size={24} />}
        </button>
      </div>

      {mobileOpen && (
        <div className="md:hidden bg-bg-primary/95 backdrop-blur-xl border-b border-border">
          <div className="px-6 py-4 flex flex-col gap-4">
            {navLinks.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="text-text-secondary hover:text-accent transition-colors py-2"
                onClick={() => setMobileOpen(false)}
              >
                {link.label}
              </a>
            ))}
            <a
              href="https://github.com/superstruct/great.sh"
              target="_blank"
              rel="noopener noreferrer"
              className="text-text-secondary hover:text-text-primary transition-colors py-2 flex items-center gap-2"
              onClick={() => setMobileOpen(false)}
            >
              <Github size={18} /> GitHub
            </a>
          </div>
        </div>
      )}
    </nav>
  )
}
