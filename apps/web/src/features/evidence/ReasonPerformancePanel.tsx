import type { ReasonPerformance } from "../../domain/evidence";

function formatPercent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${Math.round(value * 100)}%`;
}

export function ReasonPerformancePanel({ rows }: { rows: ReasonPerformance[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Similar reason performance</p>
      {rows.length === 0 ? (
        <p className="terminal-panel-copy">No publishable history for these exact reason checks has been stored yet.</p>
      ) : (
        <div className="terminal-list">
          {rows.map((row) => (
            <div className="terminal-list-row" key={`${row.reasonType}:${row.reasonKey}`}>
              <div>
                <strong>{row.reasonKey}</strong>
                <p>{row.reasonType.replaceAll("_", " ")}</p>
              </div>
              <div>
                <strong>{formatPercent(row.winRate)}</strong>
                <p>Sample size {row.sampleSize}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </article>
  );
}
