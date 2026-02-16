import Hero from '../components/Hero'
import PowerStrip from '../components/PowerStrip'
import SecurityShowcase from '../components/SecurityShowcase'
import Features from '../components/Features'
import FileFormat from '../components/FileFormat'
import Footer from '../components/Footer'

export default function LandingPage() {
  return (
    <div className="min-h-screen bg-bg-dark overflow-x-hidden">
      <Hero />
      <PowerStrip />
      <SecurityShowcase />
      <Features />
      <FileFormat />
      <Footer />
    </div>
  )
}
