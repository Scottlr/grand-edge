import { useEffect, useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  getApiClient,
  useItem,
  useItemHistory,
  useItems,
  usePositions,
  useRecommendationExplanation,
  useRecommendations,
  useStrategies,
  useToggleStrategy,
  useSimulations,
} from "../api/hooks";
import type { Item, Position, Recommendation, SimulationRun, StrategyStatus } from "../api/types";
import { createLiveConnection } from "../api/live";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { useWorkspaceStore } from "../state/workspaceStore";
import { CommandCenterView } from "../features/command-center/CommandCenterView";
import { LinkedItemsView } from "../features/linked-items/LinkedItemsView";
import { ModelAccuracyView } from "../features/model-accuracy/ModelAccuracyView";
import { StrategyLabView } from "../features/strategy-lab/StrategyLabView";
import {
  ItemIntelligenceView,
  RecommendationExplainerView,
} from "../views/DetailedViews";
import { SimulationReplayView } from "../features/simulations/SimulationReplayView";
import { AccountSettingsView } from "../features/account/AccountSettingsView";
import { PortfolioView } from "../features/portfolio/PortfolioView";
import { DataStatePanel } from "../components/state/DataStatePanel";
import type { DataState } from "../domain/recommendation";

const EMPTY_ITEMS: Item[] = [];
const EMPTY_RECOMMENDATIONS: Recommendation[] = [];
const EMPTY_STRATEGIES: StrategyStatus[] = [];
const EMPTY_POSITIONS: Position[] = [];
const EMPTY_SIMULATIONS: SimulationRun[] = [];

function deriveShellState({
  hasData,
  hasErrors,
  isLoading,
  recommendations,
}: {
  hasData: boolean;
  hasErrors: boolean;
  isLoading: boolean;
  recommendations: Recommendation[];
}): DataState {
  if (isLoading && !hasData) {
    return "loading";
  }
  if (hasErrors) {
    return "degraded";
  }
  if (recommendations[0]?.dataState === "stale") {
    return "stale";
  }
  if (recommendations[0]?.dataState === "error") {
    return "error";
  }
  if (recommendations[0]?.dataState === "degraded") {
    return "degraded";
  }
  if (!hasData) {
    return "empty";
  }

  return "live";
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
  const selectedItemQuery = useItem(selectedItemId);
  const selectedItemHistoryQuery = useItemHistory(selectedItemId, { interval: "1h", limit: 48 });
  const selectedRecommendationQuery = useRecommendationExplanation(selectedRecommendationId);
  useToggleStrategy();

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
  const selectedItem = selectedItemQuery.data ?? null;
  const selectedItemHistory = selectedItemHistoryQuery.data ?? [];

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
  const buyRecommendation = useMemo(
    () => recommendations.find((entry) => entry.action === "buy" || entry.action === "add") ?? null,
    [recommendations],
  );
  const sellRecommendation = useMemo(
    () => recommendations.find((entry) => entry.action === "cashout") ?? null,
    [recommendations],
  );
  const selectedRecommendationDetail = selectedRecommendationQuery.data ?? selectedRecommendation ?? buyRecommendation ?? null;

  const shellState = deriveShellState({
    hasData:
      items.length > 0 ||
      recommendations.length > 0 ||
      strategies.length > 0 ||
      positions.length > 0 ||
      simulations.length > 0,
    hasErrors:
      itemsQuery.isError ||
      recommendationsQuery.isError ||
      strategiesQuery.isError ||
      positionsQuery.isError ||
      simulationsQuery.isError,
    isLoading:
      itemsQuery.isLoading ||
      recommendationsQuery.isLoading ||
      strategiesQuery.isLoading ||
      positionsQuery.isLoading ||
      simulationsQuery.isLoading,
    recommendations,
  });

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
        activeView={activeView}
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
            setActiveView("items");
          }}
          onSelectView={setActiveView}
          onToggleCollapsed={toggleSidebar}
        />

        <section className="terminal-workspace">
          <DataStatePanel
            state={shellState}
            title="Shell status"
            message={shellStateMessage}
            action={
              selectedRecommendation ? (
                <p className="terminal-status-inline">
                  Selected recommendation:{" "}
                  <strong>{itemsById.get(selectedRecommendation.itemId)?.name ?? selectedRecommendation.itemId}</strong>
                </p>
              ) : undefined
            }
          />

          {activeView === "dashboard" ? (
            <CommandCenterView
              onSelectRecommendation={(recommendationId) => {
                const selected = recommendations.find((entry) => entry.recommendationId === recommendationId);
                selectRecommendation(recommendationId);
                if (selected) {
                  selectItem(selected.itemId);
                }
              }}
              positions={positions}
              recommendations={recommendations}
              selectedRecommendationId={selectedRecommendationId}
              simulations={simulations}
              strategies={strategies}
            />
          ) : null}
          {activeView === "buy" ? <RecommendationExplainerView recommendation={buyRecommendation} /> : null}
          {activeView === "sell" ? <RecommendationExplainerView recommendation={sellRecommendation} /> : null}
          {activeView === "portfolio" ? (
            <PortfolioView positions={positions} recommendations={recommendations} />
          ) : null}
          {activeView === "items" ? (
            <ItemIntelligenceView
              history={selectedItemHistory}
              item={selectedItem}
              recommendation={selectedRecommendationDetail}
            />
          ) : null}
          {activeView === "linkedItems" ? (
            <LinkedItemsView recommendation={selectedRecommendationDetail ?? selectedRecommendation ?? buyRecommendation} />
          ) : null}
          {activeView === "simulations" ? (
            <SimulationReplayView
              history={selectedItemHistory}
              recommendation={selectedRecommendationDetail}
              simulations={simulations}
            />
          ) : null}
          {activeView === "accuracy" ? (
            <ModelAccuracyView recommendation={selectedRecommendationDetail ?? selectedRecommendation ?? buyRecommendation} />
          ) : null}
          {activeView === "settings" ? (
            <>
              <AccountSettingsView />
              <StrategyLabView strategies={strategies} />
            </>
          ) : null}
        </section>
      </div>
    </main>
  );
}
