import type { ModelAccuracyViewModel } from "../../domain/modelAccuracy";

export function AccuracyFilters({
  models,
  selectedStrategyId,
  selectedWindowLabel,
  onSelectStrategy,
  onSelectWindow,
}: {
  models: ModelAccuracyViewModel[];
  selectedStrategyId: string;
  selectedWindowLabel: "7d" | "30d" | "all";
  onSelectStrategy: (strategyId: string) => void;
  onSelectWindow: (windowLabel: "7d" | "30d" | "all") => void;
}) {
  const strategyIds = [...new Set(models.map((model) => model.strategyId))];

  return (
    <article className="terminal-panel">
      <p className="eyebrow">Filters</p>
      <div className="simulation-mode-grid">
        {strategyIds.map((strategyId) => (
          <button
            className={`terminal-action-button ${selectedStrategyId === strategyId ? "terminal-nav-button-active" : ""}`}
            key={strategyId}
            type="button"
            onClick={() => onSelectStrategy(strategyId)}
          >
            {strategyId}
          </button>
        ))}
      </div>
      <div className="simulation-mode-grid">
        {(["7d", "30d", "all"] as const).map((windowLabel) => (
          <button
            className={`terminal-action-button ${selectedWindowLabel === windowLabel ? "terminal-nav-button-active" : ""}`}
            key={windowLabel}
            type="button"
            onClick={() => onSelectWindow(windowLabel)}
          >
            {windowLabel}
          </button>
        ))}
      </div>
    </article>
  );
}
