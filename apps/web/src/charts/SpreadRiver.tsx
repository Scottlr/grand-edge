import type { TimePoint } from "./scales";

export function SpreadRiver({ points }: { points: TimePoint[] }) {
  const maxSpread = Math.max(...points.map((point) => point.spread ?? 0), 1);

  return (
    <div aria-label="Spread river" className="bar-chart">
      {points.map((point) => {
        const height = point.spread === null ? 10 : Math.max(12, (point.spread / maxSpread) * 100);
        return <span className="bar-chart-bar" key={`${point.label}-spread`} style={{ height: `${height}%` }} title={`${point.label} spread`} />;
      })}
    </div>
  );
}
