import { motion } from 'framer-motion'
import { Apple, Monitor } from 'lucide-react'
import CodeBlock from './ui/CodeBlock'
import TerminalPreview from './TerminalPreview'

export default function Hero() {
  return (
    <section className="relative flex flex-col items-center justify-center px-4 py-12 sm:py-20 sm:min-h-screen overflow-hidden">
      {/* Animated grid background */}
      <div className="absolute inset-0 grid-background opacity-50" />

      {/* Gradient orbs */}
      <div className="absolute top-1/4 left-1/4 w-48 h-48 sm:w-96 sm:h-96 bg-primary/20 rounded-full blur-3xl animate-glow-pulse" />
      <div className="absolute bottom-1/4 right-1/4 w-48 h-48 sm:w-96 sm:h-96 bg-accent-cyan/20 rounded-full blur-3xl animate-glow-pulse" style={{ animationDelay: '2s' }} />

      <div className="relative z-10 w-full max-w-6xl mx-auto text-center">
        {/* Main title */}
        <motion.h1
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.1 }}
          className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-extrabold mb-4 sm:mb-6"
        >
          <span className="gradient-text">cokacenc</span>
        </motion.h1>

        {/* Tagline */}
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          className="text-xl sm:text-2xl md:text-3xl lg:text-4xl font-bold text-white mb-3 sm:mb-4"
        >
          AES-256-CBC
          <br />
          <span className="text-glow text-accent-cyan">File Encryption + Split</span>
        </motion.p>

        {/* Sub-tagline */}
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.25 }}
          className="text-sm sm:text-base md:text-lg text-zinc-400 mb-4 px-2"
        >
          Encrypt, split, and protect your files with 1-pass streaming encryption
        </motion.p>

        {/* Terminal preview */}
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 1, delay: 0.4 }}
          className="mb-8 sm:mb-16"
        >
          <TerminalPreview />
        </motion.div>

        {/* Quick Start */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
          className="w-full max-w-6xl mx-auto mb-6 sm:mb-8"
          id="quickstart"
        >
          <h2 className="text-2xl sm:text-3xl md:text-4xl font-bold mb-4">
            Get Started in <span className="text-accent-cyan">Seconds</span>
          </h2>
          <p className="text-zinc-400 text-sm sm:text-base md:text-lg mb-6 sm:mb-8">
            Three commands. That's all you need.
          </p>

          <div className="relative space-y-4">
            <div className="absolute inset-0 sm:-inset-2 bg-gradient-to-r from-primary/20 via-accent-cyan/20 to-accent-purple/20 rounded-2xl blur-xl opacity-60 pointer-events-none" />
            <div className="relative">
              <p className="text-zinc-500 text-xs text-left mb-1 ml-1">1. Generate a key file</p>
              <CodeBlock code="cokacenc generate --output secret.key" />
            </div>
            <div className="relative">
              <p className="text-zinc-500 text-xs text-left mb-1 ml-1">2. Encrypt files</p>
              <CodeBlock code="cokacenc pack --dir ./data --key secret.key" />
            </div>
            <div className="relative">
              <p className="text-zinc-500 text-xs text-left mb-1 ml-1">3. Decrypt files</p>
              <CodeBlock code="cokacenc unpack --dir ./data --key secret.key" />
            </div>
          </div>

          <div className="flex items-center justify-center gap-6 mt-8 text-zinc-500">
            <div className="flex items-center gap-2">
              <Apple className="w-4 h-4" />
              <span className="text-sm">macOS</span>
            </div>
            <div className="flex items-center gap-2">
              <Monitor className="w-4 h-4" />
              <span className="text-sm">Linux</span>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Scroll indicator */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.5 }}
        className="absolute bottom-8 left-1/2 -translate-x-1/2"
        aria-hidden="true"
      >
        <motion.div
          animate={{ y: [0, 8, 0] }}
          transition={{ duration: 1.5, repeat: Infinity }}
          className="w-6 h-10 border-2 border-zinc-600 rounded-full flex justify-center pt-2"
        >
          <div className="w-1.5 h-1.5 bg-accent-cyan rounded-full" />
        </motion.div>
      </motion.div>
    </section>
  )
}
