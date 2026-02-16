import { motion } from 'framer-motion'
import { KeyRound, Lock, Unlock, Dices, FileKey, Settings2, Split, EyeOff, Trash2, FolderSearch, Hash, FileX } from 'lucide-react'

interface SubFeature {
  icon: typeof KeyRound
  label: string
}

interface Pillar {
  title: string
  description: string
  tint: string
  borderColor: string
  iconBg: string
  subFeatures: SubFeature[]
}

const pillars: Pillar[] = [
  {
    title: 'generate',
    description: 'Generate a cryptographically secure random key file for encryption.',
    tint: 'from-accent-cyan/5 to-transparent',
    borderColor: 'border-accent-cyan/20 hover:border-accent-cyan/40',
    iconBg: 'bg-accent-cyan/10 text-accent-cyan',
    subFeatures: [
      { icon: Dices, label: 'Cryptographically secure random bytes' },
      { icon: FileKey, label: 'Base64 encoded output' },
      { icon: Settings2, label: 'Configurable key length (default 64 bytes)' },
      { icon: EyeOff, label: 'Overwrite protection (--force to override)' },
    ],
  },
  {
    title: 'pack',
    description: 'Encrypt and split files in a directory using AES-256-CBC.',
    tint: 'from-accent-purple/5 to-transparent',
    borderColor: 'border-accent-purple/20 hover:border-accent-purple/40',
    iconBg: 'bg-accent-purple/10 text-accent-purple',
    subFeatures: [
      { icon: Lock, label: 'AES-256-CBC encryption with PKCS7 padding' },
      { icon: Split, label: 'Auto-split at configurable chunk size (default 1.8GB)' },
      { icon: EyeOff, label: 'Auto-skip hidden files and .cokacenc files' },
      { icon: Trash2, label: 'Optional --delete to remove originals after encryption' },
    ],
  },
  {
    title: 'unpack',
    description: 'Decrypt and merge .cokacenc files back to their original form.',
    tint: 'from-accent-green/5 to-transparent',
    borderColor: 'border-accent-green/20 hover:border-accent-green/40',
    iconBg: 'bg-accent-green/10 text-accent-green',
    subFeatures: [
      { icon: Unlock, label: 'Auto-group and order split chunks' },
      { icon: Hash, label: 'MD5 integrity verification on every file' },
      { icon: FolderSearch, label: 'Restore original filenames automatically' },
      { icon: FileX, label: 'Optional --delete to remove .cokacenc after decryption' },
    ],
  },
]

export default function Features() {
  return (
    <section className="py-12 sm:py-24 px-4" id="features">
      <div className="max-w-6xl mx-auto">
        {/* Section header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-center mb-8 sm:mb-16"
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            <span className="gradient-text">Three Commands</span>
          </h2>
          <p className="text-zinc-400 text-sm sm:text-lg max-w-2xl mx-auto">
            Generate a key, encrypt your files, decrypt when needed. Simple.
          </p>
        </motion.div>

        {/* Pillar blocks */}
        <div className="space-y-10 sm:space-y-16">
          {pillars.map((pillar, index) => {
            const isReversed = index % 2 === 1

            return (
              <motion.div
                key={index}
                initial={{ opacity: 0, y: 30 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: 0.1 }}
                className={`flex flex-col ${isReversed ? 'lg:flex-row-reverse' : 'lg:flex-row'} gap-8 items-center`}
              >
                {/* Text side */}
                <div className="flex-1 lg:max-w-[50%]">
                  <h3 className="text-2xl sm:text-3xl font-bold mb-3 font-mono">{pillar.title}</h3>
                  <p className="text-zinc-400 mb-6">{pillar.description}</p>
                  <div className="space-y-3">
                    {pillar.subFeatures.map((sf, i) => (
                      <motion.div
                        key={i}
                        initial={{ opacity: 0, x: isReversed ? 20 : -20 }}
                        whileInView={{ opacity: 1, x: 0 }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.4, delay: 0.2 + i * 0.08 }}
                        className="flex items-center gap-3"
                      >
                        <div className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 ${pillar.iconBg}`}>
                          <sf.icon className="w-4 h-4" />
                        </div>
                        <span className="text-zinc-300 text-sm">{sf.label}</span>
                      </motion.div>
                    ))}
                  </div>
                </div>

                {/* Visual card side */}
                <div className="flex-1 lg:max-w-[50%] w-full">
                  <div className={`bg-gradient-to-br ${pillar.tint} border ${pillar.borderColor} rounded-2xl p-4 sm:p-8 transition-colors duration-300`}>
                    <PillarVisual index={index} />
                  </div>
                </div>
              </motion.div>
            )
          })}
        </div>
      </div>
    </section>
  )
}

function PillarVisual({ index }: { index: number }) {
  if (index === 0) {
    // generate: key file visualization
    return (
      <div className="font-mono text-xs space-y-2">
        <div className="border border-accent-cyan/30 rounded bg-bg-dark/50 overflow-hidden">
          <div className="bg-bg-card px-2 py-1 border-b border-zinc-800 text-[10px] text-zinc-500 flex justify-between">
            <span>$ cokacenc generate</span>
            <span className="text-accent-cyan">KeyGen</span>
          </div>
          <div className="p-2 space-y-1.5">
            <div className="text-zinc-500">$ cokacenc generate --output secret.key --length 64</div>
            <div className="text-accent-green">Generated key file: secret.key</div>
            <div className="text-zinc-600">(64 random bytes, 86 chars Base64)</div>
            <div className="mt-2 p-1.5 bg-bg-card/50 border border-zinc-800 rounded">
              <div className="text-accent-purple break-all text-[10px]">
                ra0ApkgUaY2KSxTFNWVSRTbWvNm<br />
                D3TjA3hgpwHg87/GjSi3U+6BEvR<br />
                iApY2yR/gc1H0uIghiVpOLU5aFL...
              </div>
            </div>
          </div>
        </div>
        <div className="text-center text-zinc-600 text-[10px]">Cryptographically secure random key</div>
      </div>
    )
  }

  if (index === 1) {
    // pack: encryption visualization
    return (
      <div className="font-mono text-xs space-y-2">
        <div className="border border-accent-purple/30 rounded bg-bg-dark/50 overflow-hidden">
          <div className="bg-bg-card px-2 py-1 border-b border-zinc-800 text-[10px] text-zinc-500 flex justify-between">
            <span>$ cokacenc pack</span>
            <span className="text-accent-purple">Encrypt</span>
          </div>
          <div className="p-2 space-y-1">
            <div>
              <span className="text-white">Packing: </span>
              <span className="text-zinc-400">report.pdf</span>
            </div>
            <div className="ml-2 text-accent-green">
              → 2e541.d4e90967.report.pdf.cokacenc
            </div>
            <div>
              <span className="text-white">Packing: </span>
              <span className="text-zinc-400">database.sql</span>
              <span className="text-zinc-600"> (15 MB)</span>
            </div>
            <div className="ml-2 text-yellow-400">
              → 3 chunks, MD5: a8f3bc01
            </div>
            <div className="ml-4 text-zinc-500 text-[10px]">
              77b55.SPLTD.a8f3bc01.aaaa.database.sql.cokacenc<br />
              77b55.SPLTD.a8f3bc01.aaab.database.sql.cokacenc<br />
              77b55.SPLTD.a8f3bc01.aaac.database.sql.cokacenc
            </div>
          </div>
        </div>
        <div className="text-center text-zinc-600 text-[10px]">Auto-split large files into chunks</div>
      </div>
    )
  }

  // unpack: decryption visualization
  return (
    <div className="font-mono text-xs space-y-2">
      <div className="border border-accent-green/30 rounded bg-bg-dark/50 overflow-hidden">
        <div className="bg-bg-card px-2 py-1 border-b border-zinc-800 text-[10px] text-zinc-500 flex justify-between">
          <span>$ cokacenc unpack</span>
          <span className="text-accent-green">Decrypt</span>
        </div>
        <div className="p-2 space-y-1">
          <div>
            <span className="text-white">Unpacking: </span>
            <span className="text-zinc-400">report.pdf</span>
            <span className="text-zinc-600"> (1 chunk)</span>
          </div>
          <div className="ml-2 text-accent-green">
            MD5 verified: d4e90967f0e0b45f...
          </div>
          <div>
            <span className="text-white">Unpacking: </span>
            <span className="text-zinc-400">database.sql</span>
            <span className="text-zinc-600"> (3 chunks)</span>
          </div>
          <div className="ml-2 text-accent-green">
            MD5 verified: a8f3bc01f50e0887...
          </div>
          <div className="mt-1 text-accent-green font-bold">
            All files restored successfully
          </div>
        </div>
      </div>
      <div className="text-center text-zinc-600 text-[10px]">Auto-merge chunks & verify integrity</div>
    </div>
  )
}
