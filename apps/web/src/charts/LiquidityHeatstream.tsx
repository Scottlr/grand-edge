import type { PricePoint } from "./chartTypes";
import { volumeExtent } from "./scales";

export function LiquidityHeatstream({ points }: { points: PricePoint[] }) {
  const bounds = volumeExtent(points);
  const span = bounds ? Math.max(bounds.max - bounds.min, 1) : 1;

  return (
    <div aria-label="Trading ease heatstream" className="heatstream">
      {points.map((point) => {
        const pointVolume = point.volume ?? 0;
        const intensity = bounds ? (pointVolume - bounds.min) / span : 0;
        return (
          <span
            className="heatstream-cell"
            key={`${point.label}-volume`}
            style={{ opacity: 0.2 + intensity * 0.8 }}
            title={`${point.label} observed activity ${pointVolume}`}
          />
        );
      })}
    </div>
  );
}
