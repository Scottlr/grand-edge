import type { TimePoint } from "./scales";
import { valuesExtent } from "./scales";

function toPolyline(points: TimePoint[], key: "high" | "mid" | "low", height: number) {
  const bounds = valuesExtent(points);
  if (!bounds) {
    return "";
  }

  const span = Math.max(bounds.max - bounds.min, 1);
  const step = points.length > 1 ? 100 / (points.length - 1) : 100;

  return points
    .map((point, index) => {
      const value = point[key];
      if (value === null) {
        return null;
      }
      const x = index * step;
      const y = height - ((value - bounds.min) / span) * height;
      return `${x},${y}`;
    })
    .filter((value): value is string => value !== null)
    .join(" ");
}

export function PriceEdgeRibbon({ points }: { points: TimePoint[] }) {
  return (
    <svg aria-label="Price edge ribbon" className="mini-chart" viewBox="0 0 100 60" preserveAspectRatio="none">
      <polyline className="mini-chart-line mini-chart-line-faint" fill="none" points={toPolyline(points, "high", 60)} />
      <polyline className="mini-chart-line" fill="none" points={toPolyline(points, "mid", 60)} />
      <polyline className="mini-chart-line mini-chart-line-low" fill="none" points={toPolyline(points, "low", 60)} />
    </svg>
  );
}
