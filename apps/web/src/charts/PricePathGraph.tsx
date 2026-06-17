import type { ForecastBandPoint, PricePoint, RecommendationMarkers } from "./chartTypes";
import { chartTheme } from "./chartTheme";

function priceBounds(points: PricePoint[], forecastBand: ForecastBandPoint[] | null) {
  const values = [
    ...points.flatMap((point) => [point.high, point.low, point.mid]),
    ...(forecastBand?.flatMap((point) => [point.lower, point.predicted, point.upper]) ?? []),
  ].filter((value): value is number => value !== null);
  if (values.length === 0) {
    return null;
  }

  return { min: Math.min(...values), max: Math.max(...values) };
}

function yFor(value: number, min: number, max: number, height: number) {
  const span = Math.max(max - min, 1);
  return height - ((value - min) / span) * height;
}

function polyline(points: Array<{ x: number; y: number }>) {
  return points.map((point) => `${point.x},${point.y}`).join(" ");
}

function lineFromPoints(points: PricePoint[], key: "mid" | "high" | "low", min: number, max: number) {
  const step = points.length > 1 ? 100 / (points.length - 1) : 100;
  return polyline(
    points.flatMap((point, index) => {
      const value = point[key];
      if (value === null) {
        return [];
      }

      return [{ x: index * step, y: yFor(value, min, max, 60) }];
    }),
  );
}

function forecastArea(forecastBand: ForecastBandPoint[], min: number, max: number) {
  const step = forecastBand.length > 1 ? 100 / (forecastBand.length - 1) : 100;
  const upper = forecastBand.flatMap((point, index) => {
    if (point.upper === null) {
      return [];
    }

    return [`${index * step},${yFor(point.upper, min, max, 60)}`];
  });
  const lower = [...forecastBand].reverse().flatMap((point, reverseIndex) => {
    const index = forecastBand.length - reverseIndex - 1;
    if (point.lower === null) {
      return [];
    }

    return [`${index * step},${yFor(point.lower, min, max, 60)}`];
  });

  return [...upper, ...lower].join(" ");
}

export function PricePathGraph({
  points,
  forecastBand,
  markers,
}: {
  points: PricePoint[];
  forecastBand?: ForecastBandPoint[] | null;
  markers?: RecommendationMarkers | null;
}) {
  const bounds = priceBounds(points, forecastBand ?? null);
  if (!bounds || points.length === 0) {
    return <p className="terminal-empty-state">No current price path is available yet.</p>;
  }

  return (
    <svg aria-label="Current price path graph" className="mini-chart" viewBox="0 0 100 60" preserveAspectRatio="none">
      {forecastBand && forecastBand.some((point) => point.lower !== null && point.upper !== null) ? (
        <polygon fill={chartTheme.band} points={forecastArea(forecastBand, bounds.min, bounds.max)} />
      ) : null}
      <polyline className="mini-chart-line mini-chart-line-faint" fill="none" points={lineFromPoints(points, "high", bounds.min, bounds.max)} />
      <polyline className="mini-chart-line" fill="none" points={lineFromPoints(points, "mid", bounds.min, bounds.max)} />
      <polyline className="mini-chart-line mini-chart-line-low" fill="none" points={lineFromPoints(points, "low", bounds.min, bounds.max)} />
      {markers?.entry !== null && markers?.entry !== undefined ? (
        <line className="mini-chart-marker mini-chart-marker-entry" x1="0" x2="100" y1={yFor(markers.entry, bounds.min, bounds.max, 60)} y2={yFor(markers.entry, bounds.min, bounds.max, 60)} />
      ) : null}
      {markers?.exit !== null && markers?.exit !== undefined ? (
        <line className="mini-chart-marker mini-chart-marker-exit" x1="0" x2="100" y1={yFor(markers.exit, bounds.min, bounds.max, 60)} y2={yFor(markers.exit, bounds.min, bounds.max, 60)} />
      ) : null}
      {markers?.stopLoss !== null && markers?.stopLoss !== undefined ? (
        <line className="mini-chart-marker mini-chart-marker-stop" x1="0" x2="100" y1={yFor(markers.stopLoss, bounds.min, bounds.max, 60)} y2={yFor(markers.stopLoss, bounds.min, bounds.max, 60)} />
      ) : null}
    </svg>
  );
}
