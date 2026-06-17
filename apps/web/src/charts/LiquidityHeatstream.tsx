import type { TimePoint } from "./scales";
import { volumeExtent } from "./scales";

export function LiquidityHeatstream({ points }: { points: TimePoint[] }) {
  const bounds = volumeExtent(points);
  const span = bounds ? Math.max(bounds.max - bounds.min, 1) : 1;

  return (
    <div aria-label="Liquidity heatstream" className="heatstream">
      {points.map((point) => {
        const intensity = bounds ? (point.volume - bounds.min) / span : 0;
        return (
          <span
            className="heatstream-cell"
            key={`${point.label}-volume`}
            style={{ opacity: 0.2 + intensity * 0.8 }}
            title={`${point.label} volume ${point.volume}`}
          />
        );
      })}
    </div>
  );
}
