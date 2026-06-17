import type { PricePoint } from "./chartTypes";
import { SpreadRibbonGraph } from "./SpreadRibbonGraph";

export function SpreadRiver({ points }: { points: PricePoint[] }) {
  return <SpreadRibbonGraph points={points} />;
}
