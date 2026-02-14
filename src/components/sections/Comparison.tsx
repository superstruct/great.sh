import { AnimatedSection } from '@/components/shared/AnimatedSection'
import { Container } from '@/components/layout/Container'
import { comparisonData } from '@/data/comparison'
import { Check, X } from 'lucide-react'

function CellValue({ value }: { value: boolean | string }) {
  if (value === true) return <Check size={16} className="text-accent mx-auto" />
  if (value === false) return <X size={16} className="text-text-tertiary mx-auto" />
  return <span className="text-text-secondary text-xs">{value}</span>
}

const tools = [
  { key: 'great' as const, label: 'great.sh' },
  { key: 'chezmoi' as const, label: 'chezmoi' },
  { key: 'mise' as const, label: 'mise' },
  { key: 'nix' as const, label: 'Nix' },
  { key: 'mcpm' as const, label: 'MCPM' },
  { key: 'manual' as const, label: 'Manual' },
]

export function Comparison() {
  return (
    <AnimatedSection id="compare">
      <Container>
        <h2 className="font-display text-3xl md:text-4xl text-text-primary text-center mb-4">
          How we compare
        </h2>
        <p className="text-text-secondary text-center mb-16 max-w-xl mx-auto">
          great.sh replaces the manual combination of 2&ndash;3 tools that currently takes hours to assemble.
        </p>

        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left py-3 pr-4 text-text-tertiary font-normal">Feature</th>
                {tools.map((tool) => (
                  <th
                    key={tool.key}
                    className={`py-3 px-3 text-center font-medium ${
                      tool.key === 'great' ? 'text-accent' : 'text-text-secondary'
                    }`}
                  >
                    {tool.label}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {comparisonData.map((row) => (
                <tr key={row.feature} className="border-b border-border/50">
                  <td className="py-3 pr-4 text-text-primary">{row.feature}</td>
                  {tools.map((tool) => (
                    <td
                      key={tool.key}
                      className={`py-3 px-3 text-center ${
                        tool.key === 'great' ? 'bg-accent-muted/50' : ''
                      }`}
                    >
                      <CellValue value={row[tool.key]} />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Container>
    </AnimatedSection>
  )
}
