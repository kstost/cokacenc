import { useRef } from 'react'
import { motion, useInView } from 'framer-motion'
import { Shield, KeyRound, Zap } from 'lucide-react'

const stats = [
  { icon: Shield, label: 'AES-256-CBC', sub: 'PKCS7 Padding' },
  { icon: KeyRound, label: 'PBKDF2-HMAC-SHA512', sub: '100,000 iterations' },
  { icon: Zap, label: '1-Pass Streaming', sub: 'No temp files' },
]

export default function PowerStrip() {
  const ref = useRef<HTMLDivElement>(null)
  useInView(ref, { once: true, margin: '-50px' })

  return (
    <section className="py-12 px-4">
      <div
        ref={ref}
        className="max-w-6xl mx-auto bg-bg-card/30 border border-accent-cyan/10 rounded-2xl py-10 px-6"
      >
        <div className="flex flex-col sm:flex-row items-center justify-center gap-8 sm:gap-16">
          {stats.map((stat, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: i * 0.15 }}
              className="flex items-center gap-3 text-center sm:text-left"
            >
              <stat.icon className="w-6 h-6 text-accent-cyan/60 shrink-0" />
              <div>
                <div className="text-lg sm:text-xl font-extrabold font-mono text-accent-cyan text-glow">
                  {stat.label}
                </div>
                <div className="text-zinc-400 text-sm">{stat.sub}</div>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  )
}
