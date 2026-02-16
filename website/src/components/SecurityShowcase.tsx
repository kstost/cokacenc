import { useRef } from 'react'
import { motion, useInView } from 'framer-motion'
import { Lock, Fingerprint, Shuffle, FileCheck, Layers, ShieldCheck } from 'lucide-react'

const securityFeatures = [
  { icon: Lock, text: 'AES-256-CBC with PKCS7 padding' },
  { icon: Fingerprint, text: 'PBKDF2-HMAC-SHA512 key derivation (100K iterations)' },
  { icon: Shuffle, text: 'Independent random Salt/IV per chunk' },
  { icon: FileCheck, text: 'MD5 integrity verification on decrypt' },
  { icon: Layers, text: 'Each chunk independently decryptable' },
  { icon: ShieldCheck, text: 'Same input produces different ciphertext every time' },
]

function ChunkFormatVisual() {
  return (
    <div className="relative overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-r from-accent-cyan/30 via-primary/20 to-accent-purple/30 rounded-xl blur-lg opacity-40" />
      <div className="relative bg-bg-dark border border-zinc-700 rounded-xl overflow-hidden shadow-2xl">
        {/* Title bar */}
        <div className="flex items-center gap-2 px-4 py-3 bg-bg-card border-b border-zinc-800">
          <div className="flex gap-2" aria-hidden="true">
            <div className="w-3 h-3 rounded-full bg-red-500/80" />
            <div className="w-3 h-3 rounded-full bg-yellow-500/80" />
            <div className="w-3 h-3 rounded-full bg-green-500/80" />
          </div>
          <span className="text-xs text-zinc-500 ml-2 font-mono">Chunk File Format (44B header + ciphertext)</span>
        </div>

        <div className="p-3 sm:p-5 font-mono text-xs sm:text-sm space-y-3">
          {/* Header breakdown */}
          <div className="flex flex-wrap gap-1">
            <span className="px-2 py-1 bg-accent-cyan/20 border border-accent-cyan/30 rounded text-accent-cyan">
              8B magic
            </span>
            <span className="px-2 py-1 bg-accent-purple/20 border border-accent-purple/30 rounded text-accent-purple">
              4B version
            </span>
            <span className="px-2 py-1 bg-yellow-500/20 border border-yellow-500/30 rounded text-yellow-400">
              16B PBKDF2 salt
            </span>
            <span className="px-2 py-1 bg-accent-green/20 border border-accent-green/30 rounded text-accent-green">
              16B AES IV
            </span>
            <span className="px-2 py-1 bg-zinc-500/20 border border-zinc-500/30 rounded text-zinc-400">
              ...ciphertext...
            </span>
          </div>

          {/* Hex visualization */}
          <div className="border border-zinc-800 rounded p-2 bg-bg-card/50 overflow-x-auto">
            <div className="whitespace-nowrap">
              <span className="text-accent-cyan">43 4F 4B 41 43 45 4E 43</span>
              <span className="text-zinc-600"> | </span>
              <span className="text-accent-purple">01 00 00 00</span>
              <span className="text-zinc-600"> | </span>
              <span className="text-yellow-400">a3 7f ... [salt]</span>
              <span className="text-zinc-600"> | </span>
              <span className="text-accent-green">e1 b2 ... [IV]</span>
              <span className="text-zinc-600"> | </span>
              <span className="text-zinc-500">encrypted data...</span>
            </div>
            <div className="whitespace-nowrap mt-1">
              <span className="text-accent-cyan">"COKACENC"</span>
              <span className="text-zinc-700 mx-2">───</span>
              <span className="text-accent-purple">v1</span>
              <span className="text-zinc-700 mx-2">───</span>
              <span className="text-yellow-400">random</span>
              <span className="text-zinc-700 mx-2">───</span>
              <span className="text-accent-green">random</span>
              <span className="text-zinc-700 mx-2">───</span>
              <span className="text-zinc-500">AES-256-CBC</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default function SecurityShowcase() {
  const ref = useRef<HTMLDivElement>(null)
  const inView = useInView(ref, { once: true, margin: '-100px' })

  return (
    <section className="py-12 sm:py-24 px-4 relative overflow-hidden" ref={ref}>
      {/* Background tint */}
      <div className="absolute inset-0 bg-gradient-to-b from-accent-cyan/5 via-accent-cyan/10 to-accent-cyan/5 pointer-events-none" />
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[300px] h-[300px] sm:w-[600px] sm:h-[600px] bg-accent-cyan/10 rounded-full blur-3xl pointer-events-none" />

      <div className="relative max-w-6xl mx-auto">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-8 sm:mb-16"
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Military-Grade <span className="text-accent-cyan text-glow">Encryption</span>
          </h2>
          <p className="text-zinc-400 text-sm sm:text-lg max-w-2xl mx-auto">
            Every chunk has its own random salt and IV. Same file, different ciphertext every time.
          </p>
        </motion.div>

        {/* 2-column layout */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 sm:gap-10 items-center">
          {/* Left: Chunk format visual */}
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.7 }}
          >
            <ChunkFormatVisual />
          </motion.div>

          {/* Right: Security features */}
          <div className="space-y-3 sm:space-y-4">
            {securityFeatures.map((feat, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, x: 30 }}
                animate={inView ? { opacity: 1, x: 0 } : {}}
                transition={{ duration: 0.5, delay: 0.2 + i * 0.1 }}
                className="flex items-center gap-3"
              >
                <div className="shrink-0 w-8 h-8 rounded-full bg-accent-cyan/20 border border-accent-cyan/30 flex items-center justify-center">
                  <feat.icon className="w-4 h-4 text-accent-cyan" />
                </div>
                <div className="bg-bg-card border border-zinc-800 rounded-2xl rounded-tl-sm px-4 py-3 text-sm text-zinc-300">
                  {feat.text}
                </div>
              </motion.div>
            ))}

            <motion.div
              initial={{ opacity: 0 }}
              whileInView={{ opacity: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: 0.8 }}
              className="pt-4"
            >
              <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-accent-cyan/10 border border-accent-cyan/20 text-sm text-accent-cyan">
                Built with Rust
              </span>
            </motion.div>
          </div>
        </div>
      </div>
    </section>
  )
}
