import type { DrawdownPoint } from "./chartTypes";

export function DrawdownGraph({ points }: { points: DrawdownPoint[] }) {
  const maxValue = Math.max(...points.map((point) => point.value ?? 0), 1);

  return (
    <div aria-label="Worst temporary drop graph" className="bar-chart">
      {points.map((point) => {
        const height = point.value === null ? 10 : Math.max(12, (point.value / maxValue) * 100);
        return (
          <span
            className={`bar-chart-bar bar-chart-bar-${point.status}`}
            key={point.label}
            style={{ height: `${height}%` }}
            title={`${point.label} ${point.value === null ? "skipped" : `${Math.round(point.value * 100)}%`}`}
          />
        );
      })}
    </div>
  );
}
