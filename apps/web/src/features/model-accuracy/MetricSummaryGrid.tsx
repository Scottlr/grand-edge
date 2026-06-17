import type { ModelAccuracyViewModel } from "../../domain/modelAccuracy";

function metricValue(value: number | string | null, kind: "percent" | "gp" | "text" = "text") {
  if (value === null) {
    return "Unavailable";
  }

  if (kind === "percent" && typeof value === "number") {
    return `${Math.round(value * 100)}%`;
  }
  if (kind === "gp" && typeof value === "number") {
    return `${value} gp`;
  }

  return String(value);
}

export function MetricSummaryGrid({ model }: { model: ModelAccuracyViewModel }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Metric summary</p>
      <div className="simulation-summary-grid">
        <div className="action-keynumber">
          <span className="eyebrow">Past accuracy</span>
          <strong>{metricValue(model.directionalAccuracy, "percent")}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Net simulated GP</span>
          <strong>{metricValue(model.netSimulatedGp, "gp")}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Win rate</span>
          <strong>{metricValue(model.winRate, "percent")}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Profit factor</span>
          <strong>{metricValue(model.profitFactor)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Worst temporary drop</span>
          <strong>{metricValue(model.maxDrawdown, "percent")}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Time in trade</span>
          <strong>{metricValue(model.avgTimeInTrade)}</strong>
        </div>
        <div className="action-keynumber">
          <span className="eyebrow">Capital efficiency</span>
          <strong>{metricValue(model.capitalEfficiency, "percent")}</strong>
        </div>
      </div>
    </article>
  );
}
