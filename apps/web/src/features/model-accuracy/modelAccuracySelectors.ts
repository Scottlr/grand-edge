import type { ModelAccuracyViewModel } from "../../domain/modelAccuracy";

export function selectAccuracyModel(
  models: ModelAccuracyViewModel[],
  strategyId: string,
  windowLabel: "7d" | "30d" | "all",
) {
  return (
    models.find((model) => model.strategyId === strategyId && model.windowLabel === windowLabel) ??
    models.find((model) => model.strategyId === strategyId) ??
    models[0] ??
    null
  );
}
