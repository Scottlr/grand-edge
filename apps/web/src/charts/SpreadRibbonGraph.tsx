import type { PricePoint } from "./chartTypes";

export function SpreadRibbonGraph({ points }: { points: PricePoint[] }) {
  const maxSpread = Math.max(...points.map((point) => point.spread ?? 0), 1);

  return (
    <div aria-label="Spread graph" className="bar-chart">
      {points.map((point) => {
        const height = point.spread === null ? 10 : Math.max(12, (point.spread / maxSpread) * 100);
        return (
          <span
            className="bar-chart-bar bar-chart-bar-spread"
            key={`${point.label}-spread`}
            style={{ height: `${height}%` }}
            title={`${point.label} spread ${point.spread ?? "unavailable"}`}
          />
        );
      })}
    </div>
  );
}
