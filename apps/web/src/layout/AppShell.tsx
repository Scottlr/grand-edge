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
import {
  AccuracyView,
  LinkedItemsView,
  SettingsView,
} from "../views/ActionJourneyViews";
import { CommandCenterView } from "../features/command-center/CommandCenterView";
import {
  ItemIntelligenceView,
  RecommendationExplainerView,
  TerminalPortfolioView,
} from "../views/DetailedViews";
import { SimulationReplayView } from "../features/simulations/SimulationReplayView";

const EMPTY_ITEMS: Item[] = [];
const EMPTY_RECOMMENDATIONS: Recommendation[] = [];
const EMPTY_STRATEGIES: StrategyStatus[] = [];
const EMPTY_POSITIONS: Position[] = [];
const EMPTY_SIMULATIONS: SimulationRun[] = [];

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
            <TerminalPortfolioView
              positions={positions}
              recommendation={sellRecommendation ?? selectedRecommendationDetail}
            />
          ) : null}
          {activeView === "items" ? (
            <ItemIntelligenceView
              history={selectedItemHistory}
              item={selectedItem}
              recommendation={selectedRecommendationDetail}
            />
          ) : null}
          {activeView === "linkedItems" ? <LinkedItemsView /> : null}
          {activeView === "simulations" ? (
            <SimulationReplayView
              history={selectedItemHistory}
              recommendation={selectedRecommendationDetail}
              simulations={simulations}
            />
          ) : null}
          {activeView === "accuracy" ? <AccuracyView recommendation={selectedRecommendation ?? buyRecommendation} /> : null}
          {activeView === "settings" ? <SettingsView strategies={strategies} /> : null}
        </section>
      </div>
    </main>
  );
}
