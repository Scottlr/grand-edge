import { create } from "zustand";

export type WorkspaceView =
  | "dashboard"
  | "buy"
  | "sell"
  | "portfolio"
  | "items"
  | "linkedItems"
  | "simulations"
  | "accuracy"
  | "settings";

export type LiveConnectionState = "idle" | "connecting" | "live" | "closed" | "error";

type WorkspaceState = {
  activeView: WorkspaceView;
  selectedItemId: number | null;
  selectedRecommendationId: string | null;
  sidebarCollapsed: boolean;
  strategyFiltersOpen: boolean;
  liveConnectionState: LiveConnectionState;
  setActiveView: (view: WorkspaceView) => void;
  selectItem: (itemId: number | null) => void;
  selectRecommendation: (recommendationId: string | null) => void;
  toggleSidebar: () => void;
  setStrategyFiltersOpen: (open: boolean) => void;
  setLiveConnectionState: (state: LiveConnectionState) => void;
};

export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  activeView: "dashboard",
  selectedItemId: null,
  selectedRecommendationId: null,
  sidebarCollapsed: false,
  strategyFiltersOpen: true,
  liveConnectionState: "idle",
  setActiveView: (view) => set({ activeView: view }),
  selectItem: (itemId) => set({ selectedItemId: itemId }),
  selectRecommendation: (recommendationId) => set({ selectedRecommendationId: recommendationId }),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setStrategyFiltersOpen: (open) => set({ strategyFiltersOpen: open }),
  setLiveConnectionState: (liveConnectionState) => set({ liveConnectionState }),
}));
