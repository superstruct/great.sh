import { useState } from 'react'
import { Check, Copy } from 'lucide-react'
import { cn } from '@/lib/utils'

interface CodeBlockProps {
  code: string
  language?: string
  filename?: string
  className?: string
}

export function CodeBlock({ code, language = 'bash', filename, className }: CodeBlockProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className={cn('rounded-lg overflow-hidden border border-border', className)}>
      {filename && (
        <div className="bg-bg-tertiary px-4 py-2 border-b border-border flex items-center justify-between">
          <span className="text-text-secondary text-xs font-mono">{filename}</span>
          <span className="text-text-tertiary text-xs">{language}</span>
        </div>
      )}
      <div className="relative group">
        <pre className="bg-bg-secondary p-4 overflow-x-auto">
          <code className="text-sm font-mono text-text-primary leading-relaxed">{code}</code>
        </pre>
        <button
          onClick={handleCopy}
          className={cn(
            'absolute top-3 right-3 p-2 rounded-md transition-all',
            'opacity-0 group-hover:opacity-100',
            'bg-bg-tertiary hover:bg-border text-text-secondary hover:text-text-primary',
          )}
          aria-label="Copy to clipboard"
        >
          {copied ? <Check size={14} className="text-accent" /> : <Copy size={14} />}
        </button>
      </div>
    </div>
  )
}
