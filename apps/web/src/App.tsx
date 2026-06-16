import { Activity, Database, Radar, ShieldCheck } from "lucide-react";

const panels = [
  {
    title: "Recommendation Queue",
    status: "Empty shell",
    detail: "Action cards will land after the Rust API and trust DTO tasks."
  },
  {
    title: "Market Feed",
    status: "Offline",
    detail: "No ingestion logic exists in the scaffold. This panel reserves the live slot."
  },
  {
    title: "Evidence Trail",
    status: "Planned",
    detail: "Recommendation reasons and confidence breakdowns arrive in later slices."
  }
];

export default function App() {
  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Grand Edge terminal</p>
          <h1>Operational shell</h1>
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
          <h2>Scaffold the spine, then layer in market evidence.</h2>
          <p>
            This first screen stays operational and restrained. It is a placeholder for
            recommendation, confidence, and replay surfaces owned by later tasks.
          </p>
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
            <p className="panel-status">{panel.status}</p>
            <p className="panel-detail">{panel.detail}</p>
          </article>
        ))}
      </section>
    </main>
  );
}
