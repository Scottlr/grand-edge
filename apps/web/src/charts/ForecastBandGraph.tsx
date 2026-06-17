import type { ForecastBandPoint } from "./chartTypes";
import { chartTheme } from "./chartTheme";

function bounds(points: ForecastBandPoint[]) {
  const hasVisibleBand = points.some((point) => point.lower !== null && point.upper !== null);
  if (!hasVisibleBand) {
    return null;
  }

  const values = points
    .flatMap((point) => [point.lower, point.predicted, point.upper])
    .filter((value): value is number => value !== null);
  if (values.length === 0) {
    return null;
  }

  return { min: Math.min(...values), max: Math.max(...values) };
}

function yFor(value: number, min: number, max: number, height: number) {
  const span = Math.max(max - min, 1);
  return height - ((value - min) / span) * height;
}

export function ForecastBandGraph({ points }: { points: ForecastBandPoint[] }) {
  const valueBounds = bounds(points);
  if (!valueBounds) {
    return <p className="terminal-empty-state">No likely price range is available yet.</p>;
  }

  const step = points.length > 1 ? 100 / (points.length - 1) : 100;
  const areaPoints = points.flatMap((point, index) => {
    if (point.upper === null) {
      return [];
    }

    return [`${index * step},${yFor(point.upper, valueBounds.min, valueBounds.max, 60)}`];
  });
  const lowerPoints = [...points].reverse().flatMap((point, reverseIndex) => {
    const index = points.length - reverseIndex - 1;
    if (point.lower === null) {
      return [];
    }

    return [`${index * step},${yFor(point.lower, valueBounds.min, valueBounds.max, 60)}`];
  });
  const predictedLine = points
    .flatMap((point, index) => {
      if (point.predicted === null) {
        return [];
      }

      return [`${index * step},${yFor(point.predicted, valueBounds.min, valueBounds.max, 60)}`];
    })
    .join(" ");

  return (
    <svg aria-label="Likely price range graph" className="mini-chart" viewBox="0 0 100 60" preserveAspectRatio="none">
      <polygon fill={chartTheme.band} points={[...areaPoints, ...lowerPoints].join(" ")} />
      <polyline className="mini-chart-line mini-chart-line-band" fill="none" points={predictedLine} />
    </svg>
  );
}
