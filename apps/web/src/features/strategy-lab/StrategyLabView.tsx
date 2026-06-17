import { useMemo, useState } from "react";

import { useToggleStrategy } from "../../api/hooks";
import { ActionPageHeader } from "../../views/ActionPageHeader";
import type { DataState } from "../../domain/recommendation";
import type { StrategyLabViewModel, StrategyStatus } from "../../domain/strategy";
import { toStrategyLabViewModel } from "../../domain/strategy";
import { createStrategyToggleHandler } from "./strategyLabActions";
import { StrategyDetailPanel } from "./StrategyDetailPanel";
import { strategyLabFixtureStatuses, strategyLabFixtures } from "./strategyLabFixtures";
import { StrategyTable } from "./StrategyTable";

function headerActions(labels: string[]) {
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

function headerNumbers(entries: Array<{ label: string; value: string }>) {
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

export function StrategyLabView({
  strategies,
  dataState = "live",
  toggleStrategy,
}: {
  strategies: StrategyStatus[];
  dataState?: DataState;
  toggleStrategy?: (strategyId: string, enabled: boolean) => Promise<unknown>;
}) {
  const toggleMutation = useToggleStrategy();
  const [selectedStrategyId, setSelectedStrategyId] = useState<string | null>(strategies[0]?.strategyId ?? null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const strategySource = strategies.length > 0 ? strategies : strategyLabFixtureStatuses;
  const viewModel: StrategyLabViewModel = useMemo(
    () =>
      toStrategyLabViewModel(strategySource, strategyLabFixtures, {
        dataState,
        selectedStrategyId,
        staleReason:
          dataState === "stale"
            ? "Strategy controls are read-only until fresh status data arrives."
            : null,
      }),
    [dataState, selectedStrategyId, strategySource],
  );

  const selectedRow = viewModel.rows.find((row) => row.strategyId === viewModel.selectedStrategyId) ?? null;
  const selectedDetail = selectedRow ? strategyLabFixtures[selectedRow.strategyId]?.detail ?? null : null;
  const knownStrategyIds = viewModel.rows.map((row) => row.strategyId);
  const handleToggle = createStrategyToggleHandler({
    knownStrategyIds,
    onToggle: toggleStrategy ?? (async (strategyId, enabled) => {
      await toggleMutation.mutateAsync({ strategyId, enabled });
    }),
  });

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={"HOLD"}
        confidence={null}
        why={"Strategy Lab stays behind the beginner journey. Use it to inspect methods, recent performance, and whether a control change is worth the tradeoff."}
        keyNumbers={headerNumbers([
          { label: "Methods available", value: `${viewModel.rows.length}` },
          { label: "Enabled now", value: `${viewModel.rows.filter((row) => row.enabled).length}` },
          { label: "Degraded methods", value: `${viewModel.rows.filter((row) => row.status === "degraded").length}` },
        ])}
        actions={headerActions(["Review method health", "Inspect last 10 paper bets", "Open advanced detail"])}
      />

      <article className="terminal-panel">
        <p className="eyebrow">Guardrail</p>
        <p className="terminal-panel-copy">
          This page is intentionally advanced. Dashboard, Buy, Sell, and Portfolio remain the default journey for everyday use.
        </p>
        {viewModel.staleReason ? <p className="terminal-panel-copy">{viewModel.staleReason}</p> : null}
        {errorMessage ? <p className="terminal-panel-copy">{errorMessage}</p> : null}
      </article>

      <div className="terminal-grid">
        <div className="detailed-view-stack">
          <StrategyTable
            onSelect={setSelectedStrategyId}
            onToggle={(strategyId, enabled) => {
              setErrorMessage(null);
              void handleToggle(strategyId, enabled).catch((error: unknown) => {
                setErrorMessage(error instanceof Error ? error.message : "Strategy update failed.");
              });
            }}
            pendingStrategyId={toggleMutation.variables?.strategyId ?? null}
            rows={viewModel.rows}
            selectedStrategyId={viewModel.selectedStrategyId}
          />
        </div>
        <div className="detailed-view-stack">
          <StrategyDetailPanel detail={selectedDetail} row={selectedRow} />
        </div>
      </div>
    </section>
  );
}
