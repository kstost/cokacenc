import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'

// --- Scenario A: Key Generation ---
function ScenarioGenerate() {
  const [phase, setPhase] = useState(0)

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = []
    timers.push(setTimeout(() => setPhase(1), 600))
    timers.push(setTimeout(() => setPhase(2), 1400))
    return () => timers.forEach(clearTimeout)
  }, [])

  return (
    <div className="p-3 sm:p-4 font-mono text-xs sm:text-sm">
      <div className="space-y-2">
        <div className="flex items-start gap-2">
          <span className="text-zinc-500 shrink-0">$</span>
          <span className="text-accent-cyan">cokacenc generate --output secret.key</span>
        </div>
        {phase >= 1 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="text-accent-green ml-4"
          >
            Generated key file: secret.key (64 random bytes, 86 chars Base64)
          </motion.div>
        )}
        {phase >= 2 && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="ml-4 mt-2 p-2 border border-zinc-700 rounded bg-bg-card/50"
          >
            <div className="text-zinc-500 text-[10px] mb-1">secret.key</div>
            <div className="text-accent-purple text-[10px] sm:text-xs break-all">
              ra0ApkgUaY2KSxTFNWVSRTbWvNmD3TjA3hgpwHg87/GjSi3U+6BEvRiApY2yR/gc1H0u...
            </div>
          </motion.div>
        )}
      </div>
    </div>
  )
}

// --- Scenario B: Pack (Encrypt & Split) ---
function ScenarioPack() {
  const [step, setStep] = useState(0)
  const files = [
    { name: 'document.pdf', result: '2e541.d4e90967.document.pdf.cokacenc', size: '2.4 MB' },
    { name: 'photos.zip', result: '3 chunks, MD5: a8f3bc01, total: 15.0 MB', size: '15.0 MB', split: true },
    { name: 'backup.sql', result: 'f1b3c.329d10af.backup.sql.cokacenc', size: '800 KB' },
  ]

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = []
    files.forEach((_, i) => {
      timers.push(setTimeout(() => setStep(i + 1), 600 + i * 900))
    })
    return () => timers.forEach(clearTimeout)
  }, [])

  return (
    <div className="p-3 sm:p-4 font-mono text-xs sm:text-sm">
      <div className="space-y-1.5">
        <div className="flex items-start gap-2">
          <span className="text-zinc-500 shrink-0">$</span>
          <span className="text-accent-cyan">cokacenc pack --dir ./data --key secret.key --size 5</span>
        </div>
        {files.map((f, i) => (
          step > i && (
            <motion.div
              key={i}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="ml-4 space-y-0.5"
            >
              <div>
                <span className="text-white">Packing: </span>
                <span className="text-zinc-400">{f.name}</span>
              </div>
              <div className="ml-2">
                <span className="text-zinc-500">  → </span>
                <span className={f.split ? 'text-yellow-400' : 'text-accent-green'}>{f.result}</span>
                <span className="text-zinc-600"> ({f.size})</span>
              </div>
            </motion.div>
          )
        ))}
      </div>
    </div>
  )
}

// --- Scenario C: Unpack (Decrypt & Merge) ---
function ScenarioUnpack() {
  const [step, setStep] = useState(0)
  const files = [
    { name: 'document.pdf', chunks: '1 chunk(s)', md5: 'd4e90967' },
    { name: 'photos.zip', chunks: '3 chunk(s)', md5: 'a8f3bc01' },
    { name: 'backup.sql', chunks: '1 chunk(s)', md5: '329d10af' },
  ]

  useEffect(() => {
    const timers: ReturnType<typeof setTimeout>[] = []
    files.forEach((_, i) => {
      timers.push(setTimeout(() => setStep(i + 1), 600 + i * 900))
    })
    return () => timers.forEach(clearTimeout)
  }, [])

  return (
    <div className="p-3 sm:p-4 font-mono text-xs sm:text-sm">
      <div className="space-y-1.5">
        <div className="flex items-start gap-2">
          <span className="text-zinc-500 shrink-0">$</span>
          <span className="text-accent-cyan">cokacenc unpack --dir ./data --key secret.key</span>
        </div>
        {files.map((f, i) => (
          step > i && (
            <motion.div
              key={i}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="ml-4 space-y-0.5"
            >
              <div>
                <span className="text-white">Unpacking: </span>
                <span className="text-zinc-400">{f.name}</span>
                <span className="text-zinc-600"> ({f.chunks})</span>
              </div>
              <div className="ml-2">
                <span className="text-accent-green">  MD5 verified: </span>
                <span className="text-accent-green">{f.md5}...</span>
              </div>
            </motion.div>
          )
        ))}
      </div>
    </div>
  )
}

const scenarios = [ScenarioGenerate, ScenarioPack, ScenarioUnpack]
const labels = ['Generate', 'Pack', 'Unpack']

export default function TerminalPreview() {
  const [active, setActive] = useState(0)

  useEffect(() => {
    const interval = setInterval(() => {
      setActive((prev) => (prev + 1) % scenarios.length)
    }, 5000)
    return () => clearInterval(interval)
  }, [])

  const ActiveScene = scenarios[active]

  return (
    <div className="relative w-full max-w-6xl mx-auto overflow-hidden">
      {/* Glow effect */}
      <div className="absolute inset-0 bg-gradient-to-r from-primary via-accent-cyan to-accent-purple rounded-xl blur-lg opacity-30" />

      {/* Terminal window */}
      <div className="relative bg-bg-dark border border-zinc-700 rounded-xl overflow-hidden shadow-2xl">
        {/* Title bar */}
        <div className="flex items-center justify-between px-2 sm:px-4 py-2 sm:py-3 bg-bg-card border-b border-zinc-800">
          <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
            <div className="flex gap-1.5 sm:gap-2" aria-hidden="true">
              <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-red-500/80" />
              <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-yellow-500/80" />
              <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-green-500/80" />
            </div>
            <span className="text-[10px] sm:text-xs text-zinc-500 ml-1 sm:ml-2 font-mono hidden sm:inline">cokacenc</span>
          </div>
          {/* Scene tabs */}
          <div className="flex gap-1 sm:gap-1.5">
            {labels.map((label, i) => (
              <button
                key={i}
                onClick={() => setActive(i)}
                className={`px-1 sm:px-2 py-0.5 rounded text-[8px] sm:text-[10px] font-mono transition-colors ${
                  i === active
                    ? 'bg-accent-cyan/20 text-accent-cyan'
                    : 'text-zinc-600 hover:text-zinc-400'
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {/* Content area */}
        <div className="min-h-[200px] sm:min-h-[240px]">
          <AnimatePresence mode="wait">
            <motion.div
              key={active}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.3 }}
            >
              <ActiveScene />
            </motion.div>
          </AnimatePresence>
        </div>
      </div>
    </div>
  )
}
