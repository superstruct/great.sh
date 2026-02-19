import { motion } from 'motion/react'
import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface AnimatedSectionProps {
  id: string
  children: ReactNode
  className?: string
}

export function AnimatedSection({ id, children, className }: AnimatedSectionProps) {
  return (
    <motion.section
      id={id}
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: '-100px' }}
      transition={{ duration: 0.5, ease: 'easeOut' }}
      className={cn('py-24 md:py-32', className)}
    >
      {children}
    </motion.section>
  )
}
