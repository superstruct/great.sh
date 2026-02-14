import { cn } from '@/lib/utils'
import type { ReactNode } from 'react'

interface TerminalWindowProps {
  title?: string
  children: ReactNode
  className?: string
}

export function TerminalWindow({ title = 'Terminal', children, className }: TerminalWindowProps) {
  return (
    <div className={cn('rounded-xl overflow-hidden border border-border shadow-2xl', className)}>
      <div className="bg-bg-tertiary px-4 py-3 flex items-center gap-2">
        <div className="flex gap-1.5">
          <div className="w-3 h-3 rounded-full bg-red-brand/80" />
          <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
          <div className="w-3 h-3 rounded-full bg-accent/80" />
        </div>
        <span className="text-text-tertiary text-xs font-mono ml-2">{title}</span>
      </div>
      <div className="bg-bg-secondary p-4 font-mono text-sm leading-relaxed">
        {children}
      </div>
    </div>
  )
}
