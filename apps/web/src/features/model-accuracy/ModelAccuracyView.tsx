import { useMemo, useState } from "react";

import type { Recommendation } from "../../api/types";
import { ActionPageHeader } from "../../views/ActionPageHeader";
import { accuracyStatusForModel } from "../../domain/modelAccuracy";
import { ExpandableAdvancedPanel } from "../../components/disclosure/ExpandableAdvancedPanel";
import { AccuracyStatus } from "./AccuracyStatus";
import { AccuracyFilters } from "./AccuracyFilters";
import { CalibrationPanel } from "./CalibrationPanel";
import { MetricSummaryGrid } from "./MetricSummaryGrid";
import { modelAccuracyFixtures, trustSummaryFixture } from "./modelAccuracyFixtures";
import { selectAccuracyModel } from "./modelAccuracySelectors";
import { TrustSummary } from "./TrustSummary";

function detailActions(labels: string[]) {
  return (
    <>
      {labels.map((label) => (
        <button className="terminal-action-button" key={label} type="button">
          {label}
        </button>
      ))}
    </>
  );
}

function detailNumbers(entries: Array<{ label: string; value: string }>) {
  return (
    <>
      {entries.map((entry) => (
        <div className="action-keynumber" key={entry.label}>
          <span className="eyebrow">{entry.label}</span>
          <strong>{entry.value}</strong>
        </div>
      ))}
    </>
  );
}

function formatPercent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }

  return `${Math.round(value * 100)}%`;
}

function formatNumber(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }

  return `${value}`;
}

export function ModelAccuracyView({
  recommendation,
  initialAdvancedOpen = false,
}: {
  recommendation: Recommendation | null;
  initialAdvancedOpen?: boolean;
}) {
  const allModels = useMemo(() => modelAccuracyFixtures, []);
  const [selectedStrategyId, setSelectedStrategyId] = useState(allModels[0]?.strategyId ?? "");
  const [selectedWindowLabel, setSelectedWindowLabel] = useState<"7d" | "30d" | "all">("7d");
  const [advancedOpen, setAdvancedOpen] = useState(initialAdvancedOpen);

  const selectedModel = selectAccuracyModel(allModels, selectedStrategyId, selectedWindowLabel);

  if (!selectedModel) {
    return null;
  }

  const status = accuracyStatusForModel(selectedModel);

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={recommendation ? "WATCH CLOSELY" : "WATCH CLOSELY"}
        confidence={recommendation?.recommendationConfidence ?? null}
        why={"Can I trust this? Start with confidence honesty and past outcomes before technical model detail."}
        keyNumbers={detailNumbers([
          {
            label: "Past accuracy",
            value: selectedModel.directionalAccuracy === null ? "Unavailable" : `${Math.round(selectedModel.directionalAccuracy * 100)}%`,
          },
          { label: "Sample size", value: `${selectedModel.sampleSize}` },
          { label: "Status", value: status.replaceAll("_", " ") },
        ])}
        actions={detailActions(["Review confidence honesty", "Compare methods", "Open advanced detail"])}
      />

      <TrustSummary summary={trustSummaryFixture} />

      <div className="terminal-grid">
        <div className="detailed-view-stack">
          <MetricSummaryGrid model={selectedModel} />
          <CalibrationPanel model={selectedModel} />
          <AccuracyStatus model={selectedModel} />
        </div>

        <div className="detailed-view-stack">
          <AccuracyFilters
            models={allModels}
            selectedStrategyId={selectedModel.strategyId}
            selectedWindowLabel={selectedWindowLabel}
            onSelectStrategy={setSelectedStrategyId}
            onSelectWindow={setSelectedWindowLabel}
          />
          <article className="terminal-panel">
            <p className="eyebrow">Advanced detail</p>
            <p className="terminal-panel-copy">
              Technical scoring stays secondary. Open this only when you want method-specific calibration detail.
            </p>
            <button className="terminal-action-button" type="button" onClick={() => setAdvancedOpen((open) => !open)}>
              {advancedOpen ? "Hide advanced detail" : "Open advanced detail"}
            </button>
            {advancedOpen ? (
              <ExpandableAdvancedPanel defaultOpen title="Advanced method detail">
                <dl className="advanced-definition-list">
                  <dt>Brier score</dt>
                  <dd>{formatNumber(selectedModel.brierScore)}</dd>
                  <dt>Model version</dt>
                  <dd>{selectedModel.modelVersion}</dd>
                  <dt>Window</dt>
                  <dd>{selectedModel.windowLabel}</dd>
                  <dt>Past accuracy</dt>
                  <dd>{formatPercent(selectedModel.directionalAccuracy)}</dd>
                </dl>
              </ExpandableAdvancedPanel>
            ) : null}
          </article>
          <article className="terminal-panel">
            <p className="eyebrow">Method summary</p>
            <div className="simulation-summary-grid">
              <div className="action-keynumber">
                <span className="eyebrow">What it does</span>
                <strong>{selectedModel.whatItDoes}</strong>
              </div>
              <div className="action-keynumber">
                <span className="eyebrow">Works best when</span>
                <strong>{selectedModel.worksBestWhen}</strong>
              </div>
              <div className="action-keynumber">
                <span className="eyebrow">Weakness</span>
                <strong>{selectedModel.weakness}</strong>
              </div>
            </div>
          </article>
        </div>
      </div>
    </section>
  );
}
