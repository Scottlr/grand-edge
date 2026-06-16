import type { CSSProperties } from "react";
import { Activity, Database, Radar, ShieldCheck } from "lucide-react";
import { actionTones } from "./design/actions";
import { accessibility } from "./design/accessibility";
import { layout } from "./design/layout";
import { motion } from "./design/motion";
import { typography } from "./design/typography";

const panels = [
  {
    title: "Recommendation Queue",
    status: "Empty shell",
    statusClassName: "panel-status-empty",
    detail: "Action cards will land after the Rust API and trust DTO tasks."
  },
  {
    title: "Market Feed",
    status: "Offline",
    statusClassName: "panel-status-live",
    detail: "No ingestion logic exists in the scaffold. This panel reserves the live slot."
  },
  {
    title: "Evidence Trail",
    status: "Planned",
    statusClassName: "panel-status-planned",
    detail: "Recommendation reasons and confidence breakdowns arrive in later slices."
  }
];

export default function App() {
  const shellStyle = {
    "--app-ui-font": typography.uiFont,
    "--app-mono-font": typography.monoFont,
    "--app-letter-spacing": typography.letterSpacing,
    "--app-workspace-max-width": layout.workspaceMaxWidth,
    "--app-topbar-height": layout.topBarHeight,
    "--app-min-touch-target": accessibility.minInteractiveSizePx,
    "--app-card-radius": layout.cardRadius,
    "--app-control-radius": layout.controlRadius,
    "--app-focus-outline": accessibility.focusOutlinePx,
    "--app-focus-offset": accessibility.focusOffsetPx,
    "--app-motion-micro": `${motion.microMs}ms`,
    "--app-motion-panel": `${motion.panelMs}ms`,
    "--app-easing-reveal": motion.easingReveal,
    "--app-easing-panel": motion.easingPanel
  } as CSSProperties;

  const toneSequence = [
    actionTones.buy,
    actionTones.sell,
    actionTones.wait,
    actionTones.hold,
    actionTones.avoid
  ];

  return (
    <main className="app-shell" style={shellStyle}>
      <header className="topbar">
        <div className="topbar-copy">
          <p className="eyebrow">Grand Edge terminal</p>
          <h1>Operational shell</h1>
          <p>
            Calm, dark, and audit-first. This shell now carries the shared color,
            motion, typography, and action-tone rules that later recommendation views
            should consume instead of inventing.
          </p>
        </div>
        <div className="status-strip" aria-label="scaffold status">
          <span>
            <Database size={16} />
            Postgres optional
          </span>
          <span>
            <Activity size={16} />
            API placeholder
          </span>
          <span>
            <ShieldCheck size={16} />
            Paper trading only
          </span>
        </div>
      </header>

      <section className="hero-grid">
        <article className="hero-card">
          <div className="hero-label">
            <Radar size={18} />
            Next action
          </div>
          <p className="hero-kicker">Design foundation</p>
          <h2>Scaffold the spine, then layer in market evidence.</h2>
          <p>
            This first screen stays operational and restrained. It is a placeholder for
            recommendation, confidence, and replay surfaces owned by later tasks.
          </p>
          <div className="tone-row" aria-label="action tones">
            {toneSequence.map((tone) => (
              <span
                className="tone-chip"
                key={tone.label}
                style={{ "--tone-color": tone.cssVar } as CSSProperties}
              >
                {tone.label}
              </span>
            ))}
          </div>
        </article>

        <article className="hero-card hero-card-muted">
          <p className="metric-label">Live mode</p>
          <p className="metric-value">Not connected</p>
          <p className="metric-detail">
            The UI shell builds independently from the Rust workspace and does not imply
            live market coverage yet.
          </p>
        </article>
      </section>

      <section className="panel-grid">
        {panels.map((panel) => (
          <article className="panel" key={panel.title}>
            <p className="panel-title">{panel.title}</p>
            <p className={`panel-status ${panel.statusClassName}`}>{panel.status}</p>
            <p className="panel-detail">{panel.detail}</p>
          </article>
        ))}
      </section>
    </main>
  );
}
