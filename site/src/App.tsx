import { Nav } from '@/components/layout/Nav'
import { Footer } from '@/components/layout/Footer'
import { Hero } from '@/components/sections/Hero'
import { Features } from '@/components/sections/Features'
import { Config } from '@/components/sections/Config'
import { HowItWorks } from '@/components/sections/HowItWorks'
import { Loop } from '@/components/sections/Loop'
import { Bridge } from '@/components/sections/Bridge'
import { Templates } from '@/components/sections/Templates'
import { Comparison } from '@/components/sections/Comparison'
import { OpenSource } from '@/components/sections/OpenSource'

export function App() {
  return (
    <div className="min-h-screen bg-bg-primary">
      <Nav />
      <main>
        <Hero />
        <Features />
        <Config />
        <HowItWorks />
        <Loop />
        <Bridge />
        <Templates />
        <Comparison />
        <OpenSource />
      </main>
      <Footer />
    </div>
  )
}
