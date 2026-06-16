import { useEffect, useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";

import { getApiClient, useItems, usePositions, useRecommendations, useStrategies, useToggleStrategy, useSimulations } from "../api/hooks";
import type { Item, Position, Recommendation, SimulationRun, StrategyStatus } from "../api/types";
import { createLiveConnection } from "../api/live";
import { Sidebar } from "./Sidebar";
import { TerminalGrid } from "./TerminalGrid";
import { TopBar } from "./TopBar";
import { useWorkspaceStore } from "../state/workspaceStore";

const EMPTY_ITEMS: Item[] = [];
const EMPTY_RECOMMENDATIONS: Recommendation[] = [];
const EMPTY_STRATEGIES: StrategyStatus[] = [];
const EMPTY_POSITIONS: Position[] = [];
const EMPTY_SIMULATIONS: SimulationRun[] = [];

function activeViewLabel(activeView: ReturnType<typeof useWorkspaceStore.getState>["activeView"]) {
  switch (activeView) {
    case "item":
      return "Item focus";
    case "strategies":
      return "Strategies";
    case "portfolio":
      return "Portfolio";
    case "simulations":
      return "Simulations";
    case "explainer":
      return "Explainer";
    case "settings":
      return "Settings";
    case "command":
    default:
      return "Command center";
  }
}

export function AppShell() {
  const queryClient = useQueryClient();
  const {
    activeView,
    liveConnectionState,
    selectedItemId,
    selectedRecommendationId,
    setActiveView,
    selectItem,
    selectRecommendation,
    sidebarCollapsed,
    setLiveConnectionState,
    toggleSidebar,
  } = useWorkspaceStore();

  const itemsQuery = useItems({ limit: 24, offset: 0 });
  const recommendationsQuery = useRecommendations({ limit: 24, offset: 0 });
  const strategiesQuery = useStrategies();
  const positionsQuery = usePositions();
  const simulationsQuery = useSimulations({ limit: 10, offset: 0 });
  const toggleStrategy = useToggleStrategy();

  useEffect(() => {
    const connection = createLiveConnection(queryClient, getApiClient().liveUrl(), {
      onStatusChange: setLiveConnectionState,
    });
    return () => {
      connection.close();
    };
  }, [queryClient, setLiveConnectionState]);

  const items = itemsQuery.data ?? EMPTY_ITEMS;
  const recommendations = recommendationsQuery.data ?? EMPTY_RECOMMENDATIONS;
  const strategies = strategiesQuery.data ?? EMPTY_STRATEGIES;
  const positions = positionsQuery.data ?? EMPTY_POSITIONS;
  const simulations = simulationsQuery.data ?? EMPTY_SIMULATIONS;

  const itemsById = useMemo(
    () =>
      new Map(
        items.map((item) => [
          item.itemId,
          {
            name: item.name,
            iconUrl: item.icon?.cdnUrl ?? null,
          },
        ]),
      ),
    [items],
  );

  const selectedRecommendation = useMemo(
    () => recommendations.find((entry) => entry.recommendationId === selectedRecommendationId) ?? null,
    [recommendations, selectedRecommendationId],
  );

  const shellStateMessage =
    itemsQuery.isError ||
    recommendationsQuery.isError ||
    strategiesQuery.isError ||
    positionsQuery.isError ||
    simulationsQuery.isError
      ? "API data is currently degraded. The terminal shell remains usable and shows backend error states instead of inventing data."
      : "Backend data is available for items, recommendations, strategies, positions, and simulations.";

  return (
    <main className="terminal-shell">
      <TopBar
        liveConnectionState={liveConnectionState}
        recommendationCount={recommendations.length}
        simulationCount={simulations.length}
        strategyCount={strategies.length}
      />

      <div className="terminal-shell-body">
        <Sidebar
          activeView={activeView}
          collapsed={sidebarCollapsed}
          items={items}
          selectedItemId={selectedItemId}
          onSelectItem={(itemId) => {
            selectItem(itemId);
            setActiveView("item");
          }}
          onSelectView={setActiveView}
          onToggleCollapsed={toggleSidebar}
        />

        <section className="terminal-workspace">
          <article className="terminal-panel terminal-status-banner">
            <p className="eyebrow">Shell status</p>
            <p>{shellStateMessage}</p>
            {selectedRecommendation ? (
              <p className="terminal-status-inline">
                Selected recommendation:{" "}
                <strong>{itemsById.get(selectedRecommendation.itemId)?.name ?? selectedRecommendation.itemId}</strong>
              </p>
            ) : null}
          </article>

          <TerminalGrid
            activeViewLabel={activeViewLabel(activeView)}
            itemsById={itemsById}
            onSelectRecommendation={(recommendationId, itemId) => {
              selectRecommendation(recommendationId);
              selectItem(itemId);
              setActiveView("explainer");
            }}
            onToggleStrategy={(strategyId, enabled) => {
              void toggleStrategy.mutateAsync({ strategyId, enabled });
            }}
            positions={positions}
            recommendations={recommendations}
            simulations={simulations}
            strategies={strategies}
            strategyMutationPendingId={toggleStrategy.variables?.strategyId ?? null}
          />
        </section>
      </div>
    </main>
  );
}
