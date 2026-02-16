import { motion, useInView } from 'framer-motion'
import { useRef } from 'react'

export default function FileFormat() {
  const ref = useRef<HTMLDivElement>(null)
  const inView = useInView(ref, { once: true, margin: '-100px' })

  return (
    <section className="py-12 sm:py-24 px-4" ref={ref}>
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-8 sm:mb-16"
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            How It <span className="text-accent-cyan text-glow">Works</span>
          </h2>
          <p className="text-zinc-400 text-sm sm:text-lg max-w-2xl mx-auto">
            1-pass streaming encryption. Read once, write once. No temporary files.
          </p>
        </motion.div>

        {/* Workflow visual */}
        <div className="grid grid-cols-1 md:grid-cols-[1fr_auto_1fr] gap-8 items-center mb-12 sm:mb-20">
          {/* Left: Pack workflow */}
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            animate={inView ? { opacity: 1, x: 0 } : {}}
            transition={{ duration: 0.6 }}
            className="space-y-3"
          >
            <h3 className="text-lg font-bold text-accent-cyan mb-4 font-mono">pack</h3>
            {[
              { step: '1', text: 'Read file in 64KB buffers', color: 'text-zinc-400' },
              { step: '2', text: 'Compute MD5 hash + AES-256-CBC encrypt', color: 'text-accent-purple' },
              { step: '3', text: 'Split into chunks at --size boundary', color: 'text-yellow-400' },
              { step: '4', text: 'Rename with content MD5 prefix', color: 'text-accent-green' },
              { step: '5', text: '--delete: remove original file', color: 'text-zinc-500' },
            ].map((item, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, x: -20 }}
                animate={inView ? { opacity: 1, x: 0 } : {}}
                transition={{ duration: 0.4, delay: i * 0.1 }}
                className="flex items-center gap-3"
              >
                <span className="w-6 h-6 rounded-full bg-accent-cyan/20 border border-accent-cyan/30 flex items-center justify-center text-xs text-accent-cyan font-mono shrink-0">
                  {item.step}
                </span>
                <span className={`text-sm ${item.color}`}>{item.text}</span>
              </motion.div>
            ))}
          </motion.div>

          {/* Center: arrow */}
          <motion.div
            initial={{ opacity: 0, scale: 0.5 }}
            animate={inView ? { opacity: 1, scale: 1 } : {}}
            transition={{ duration: 0.6, delay: 0.5 }}
            className="flex justify-center"
          >
            <div className="flex flex-col items-center gap-2">
              <div className="hidden md:flex flex-col items-center gap-1">
                <svg width="48" height="24" viewBox="0 0 48 24" fill="none" className="text-accent-cyan rotate-180">
                  <path d="M0 12H40M40 12L32 4M40 12L32 20" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
                <span className="text-zinc-600 text-[10px] font-mono">.cokacenc</span>
                <svg width="48" height="24" viewBox="0 0 48 24" fill="none" className="text-accent-green">
                  <path d="M0 12H40M40 12L32 4M40 12L32 20" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
              <div className="block md:hidden">
                <svg width="24" height="48" viewBox="0 0 24 48" fill="none" className="text-accent-cyan">
                  <path d="M12 0V40M12 40L4 32M12 40L20 32" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
            </div>
          </motion.div>

          {/* Right: Unpack workflow */}
          <motion.div
            initial={{ opacity: 0, x: 30 }}
            animate={inView ? { opacity: 1, x: 0 } : {}}
            transition={{ duration: 0.6, delay: 0.3 }}
            className="space-y-3"
          >
            <h3 className="text-lg font-bold text-accent-green mb-4 font-mono">unpack</h3>
            {[
              { step: '1', text: 'Scan directory for .cokacenc files', color: 'text-zinc-400' },
              { step: '2', text: 'Group chunks by original filename', color: 'text-accent-cyan' },
              { step: '3', text: 'Decrypt each chunk in sequence order', color: 'text-accent-purple' },
              { step: '4', text: 'Merge into single file + compute MD5', color: 'text-yellow-400' },
              { step: '5', text: 'Verify MD5 matches filename prefix', color: 'text-accent-green' },
            ].map((item, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, x: 20 }}
                animate={inView ? { opacity: 1, x: 0 } : {}}
                transition={{ duration: 0.4, delay: 0.3 + i * 0.1 }}
                className="flex items-center gap-3"
              >
                <span className="w-6 h-6 rounded-full bg-accent-green/20 border border-accent-green/30 flex items-center justify-center text-xs text-accent-green font-mono shrink-0">
                  {item.step}
                </span>
                <span className={`text-sm ${item.color}`}>{item.text}</span>
              </motion.div>
            ))}
          </motion.div>
        </div>

        {/* File naming convention */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="max-w-4xl mx-auto"
        >
          <div className="relative">
            <div className="absolute inset-0 bg-gradient-to-r from-primary/20 via-accent-cyan/20 to-accent-purple/20 rounded-2xl blur-2xl" />
            <div className="relative bg-bg-card border border-zinc-800 rounded-2xl p-6 sm:p-8">
              <h3 className="text-lg font-bold mb-4 text-center">Output Filename Convention</h3>

              {/* Single file */}
              <div className="mb-4">
                <div className="text-zinc-500 text-xs mb-2">Single file (no split):</div>
                <div className="font-mono text-xs sm:text-sm bg-bg-dark/50 border border-zinc-800 rounded p-3 overflow-x-auto">
                  <span className="text-accent-cyan">2e541</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-accent-green">d4e90967</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-white">report.pdf</span>
                  <span className="text-zinc-500">.cokacenc</span>
                </div>
                <div className="flex flex-wrap gap-x-4 gap-y-1 mt-2 text-[10px]">
                  <span className="text-accent-cyan">fnMD5 (5 chars)</span>
                  <span className="text-accent-green">contentMD5 (8 chars)</span>
                  <span className="text-white">original name</span>
                </div>
              </div>

              {/* Split file */}
              <div>
                <div className="text-zinc-500 text-xs mb-2">Split file:</div>
                <div className="font-mono text-xs sm:text-sm bg-bg-dark/50 border border-zinc-800 rounded p-3 overflow-x-auto">
                  <span className="text-accent-cyan">77b55</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-yellow-400">SPLTD</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-accent-green">a8f3bc01</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-accent-purple">aaaa</span>
                  <span className="text-zinc-600">.</span>
                  <span className="text-white">database.sql</span>
                  <span className="text-zinc-500">.cokacenc</span>
                </div>
                <div className="flex flex-wrap gap-x-4 gap-y-1 mt-2 text-[10px]">
                  <span className="text-accent-cyan">fnMD5</span>
                  <span className="text-yellow-400">SPLTD marker</span>
                  <span className="text-accent-green">contentMD5</span>
                  <span className="text-accent-purple">seq (aaaa~zzzz, 456,976 max)</span>
                  <span className="text-white">original name</span>
                </div>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  )
}
