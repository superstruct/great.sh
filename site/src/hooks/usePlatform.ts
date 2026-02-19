import { useState, useEffect } from 'react'

export type Platform = 'macos' | 'linux' | 'windows' | 'unknown'

export function usePlatform(): Platform {
  const [platform, setPlatform] = useState<Platform>('unknown')

  useEffect(() => {
    const ua = navigator.userAgent.toLowerCase()
    if (ua.includes('mac')) {
      setPlatform('macos')
    } else if (ua.includes('win')) {
      setPlatform('windows')
    } else if (ua.includes('linux')) {
      setPlatform('linux')
    }
  }, [])

  return platform
}
