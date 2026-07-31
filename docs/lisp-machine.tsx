// pages/lisp-machine.tsx — public landing page for the LISP Machine
import Head from 'next/head'
import Link from 'next/link'
import PublicLayout from '../components/layout/PublicLayout'
import type { NextPageWithLayout } from './_app'
import type { ReactElement } from 'react'

const BG     = '#040408'
const CARD   = '#0a0a0f'
const BORDER = 'rgba(255,255,255,0.07)'
const TEXT   = '#f5f5f7'
const MUTED  = 'rgba(255,255,255,0.45)'
const GREEN  = '#00ff88'
const PURPLE = '#a78bfa'
const ORANGE = '#f97316'
const GOLD   = '#ffd60a'
const MONO   = "'Space Mono', monospace"
const UI     = "-apple-system, 'Inter', 'Helvetica Neue', sans-serif"

const LISPMachinePage: NextPageWithLayout = () => (
  <PublicLayout>
    <Head>
      <title>LISP Machine — SnapKitty Sovereign OS</title>
      <meta name="description" content="A virtual LISP machine with tagged memory, agent heap, WORM-sealed world dumps, and a reflective OS. The architecture that solves the AI soul and context problem permanently." />
    </Head>

    <div style={{ background: BG, color: TEXT, fontFamily: UI, minHeight: '100vh' }}>
      <div style={{ maxWidth: 1100, margin: '0 auto', padding: '64px 24px 96px' }}>

        {/* Hero */}
        <div style={{ marginBottom: 64 }}>
          <div style={{ fontFamily: MONO, fontSize: 10, color: PURPLE, letterSpacing: '0.2em', marginBottom: 12 }}>
            LISP MACHINE · SOVEREIGN VIRTUAL RUNTIME
          </div>
          <h1 style={{ fontSize: 'clamp(30px,5vw,52px)', fontWeight: 700, letterSpacing: '-0.03em', margin: '0 0 20px', lineHeight: 1.1 }}>
            The machine that<br />never forgets.
          </h1>
          <p style={{ fontSize: 16, color: MUTED, maxWidth: 600, lineHeight: 1.8, margin: '0 0 16px' }}>
            The SnapKitty LISP Machine is a virtual environment with tagged memory, a sovereign agent heap, WORM-sealed world dumps, and a reflective OS. It is the architecture that solves the AI soul and context problem — permanently.
          </p>
          <p style={{ fontSize: 14, color: MUTED, maxWidth: 560, lineHeight: 1.8, margin: '0 0 36px' }}>
            When the world dump is sealed, the machine's entire state — every symbol, every binding, every agent decision — is anchored to the WORM chain. Context never dies.
          </p>
          <div style={{ display: 'flex', gap: 14, flexWrap: 'wrap' }}>
            <Link href="/contact" style={{
              fontFamily: MONO, fontSize: 11, letterSpacing: '0.1em', fontWeight: 700,
              color: '#001b0e', background: GREEN, padding: '13px 28px',
              borderRadius: 6, textDecoration: 'none',
            }}>
              JOIN WAITLIST →
            </Link>
            <Link href="/papers" style={{
              fontFamily: MONO, fontSize: 11, letterSpacing: '0.1em',
              color: TEXT, padding: '13px 28px', borderRadius: 6,
              textDecoration: 'none', border: `1px solid ${BORDER}`,
            }}>
              READ THE PAPERS
            </Link>
          </div>
        </div>

        {/* Architecture phases */}
        <div style={{ marginBottom: 64 }}>
          <div style={{ fontFamily: MONO, fontSize: 10, color: MUTED, letterSpacing: '0.2em', marginBottom: 28 }}>BUILD PHASES</div>
          <div style={{ display: 'grid', gap: 12 }}>
            {[
              { phase: '01', label: 'Tagged Memory Architecture', color: GREEN,
                desc: 'Every heap cell carries a type tag, ownership tag, and WORM sequence number. The borrow checker enforces single-ownership at the interpreter level — no aliasing, no drift.' },
              { phase: '02', label: 'Agent Heap', color: PURPLE,
                desc: 'Each sovereign agent owns a named region of the heap. Agents communicate via message passing — no shared mutable state. The heap is inspectable and WORM-logged.' },
              { phase: '03', label: 'Reflective Evaluator', color: ORANGE,
                desc: 'The evaluator can inspect and rewrite itself. New agent personas, new axioms, new execution rules — all expressible as LISP forms and sealed before activation.' },
              { phase: '04', label: 'WORM World Dumps', color: GOLD,
                desc: 'At any point the machine can snapshot its entire world — all symbols, all bindings, all agent state — into a SHA-256-sealed WORM entry. Load it on any machine. State survives restarts.' },
              { phase: '05', label: 'Sovereign Boot', color: GREEN,
                desc: 'The boot sequence loads from the last verified world dump. The machine re-enters exactly where it left off. No cold start. No amnesia. The agent remembers everything.' },
            ].map(p => (
              <div key={p.phase} style={{
                background: CARD, border: `1px solid ${BORDER}`,
                borderRadius: 10, padding: '22px 24px',
                display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0 24px', alignItems: 'start',
              }}>
                <div style={{ fontFamily: MONO, fontSize: 28, fontWeight: 700, color: p.color, lineHeight: 1, paddingTop: 2 }}>{p.phase}</div>
                <div>
                  <div style={{ fontSize: 15, fontWeight: 700, color: p.color, marginBottom: 8 }}>{p.label}</div>
                  <div style={{ fontSize: 13, color: MUTED, lineHeight: 1.75 }}>{p.desc}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Why LISP */}
        <div style={{
          background: `rgba(167,139,250,0.05)`, border: `1px solid rgba(167,139,250,0.2)`,
          borderRadius: 12, padding: '36px 32px', marginBottom: 64,
        }}>
          <div style={{ fontFamily: MONO, fontSize: 10, color: PURPLE, letterSpacing: '0.2em', marginBottom: 20 }}>WHY LISP</div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 32 }}>
            <div>
              <div style={{ fontSize: 15, fontWeight: 700, marginBottom: 12 }}>Code is data. Data is memory.</div>
              <div style={{ fontSize: 13, color: MUTED, lineHeight: 1.8 }}>
                In a LISP machine, agent instructions and agent memory live in the same heap. An agent can inspect its own decision history, rewrite its own evaluation rules, and seal the result — all in one expression. No other architecture makes this possible without a separate reflection layer.
              </div>
            </div>
            <div>
              <div style={{ fontSize: 15, fontWeight: 700, marginBottom: 12 }}>The soul problem — solved.</div>
              <div style={{ fontSize: 13, color: MUTED, lineHeight: 1.8 }}>
                Every AI has amnesia between sessions. The LISP Machine seals world dumps to the WORM chain. Reload the dump and the agent has perfect recall of every prior decision, every prior seal, every prior instruction. The context window becomes infinite — it is the WORM chain itself.
              </div>
            </div>
          </div>
        </div>

        {/* Status + CTA */}
        <div style={{
          background: CARD, border: `1px solid ${BORDER}`,
          borderRadius: 12, padding: '40px 36px',
          display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 24,
        }}>
          <div>
            <div style={{ fontFamily: MONO, fontSize: 9, color: ORANGE, letterSpacing: '0.15em', marginBottom: 10 }}>STATUS: IN BUILD</div>
            <div style={{ fontSize: 20, fontWeight: 700, marginBottom: 8 }}>Phase 1–2 underway.</div>
            <div style={{ fontSize: 13, color: MUTED }}>Join the waitlist to get early access and follow the build in public.</div>
          </div>
          <Link href="/contact" style={{
            fontFamily: MONO, fontSize: 11, letterSpacing: '0.1em', fontWeight: 700,
            color: '#001b0e', background: GREEN, padding: '14px 28px',
            borderRadius: 6, textDecoration: 'none', whiteSpace: 'nowrap',
          }}>
            JOIN WAITLIST →
          </Link>
        </div>

      </div>
    </div>
  </PublicLayout>
)

LISPMachinePage.getLayout = (page: ReactElement) => <PublicLayout>{page}</PublicLayout>

export default LISPMachinePage
