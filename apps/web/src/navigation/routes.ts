import {
  BadgeDollarSign,
  ChartColumnBig,
  ChartLine,
  FolderKanban,
  LayoutDashboard,
  Link2,
  Settings,
  Shield,
  ShoppingCart,
} from "lucide-react";

import type { WorkspaceView } from "../state/workspaceStore";

export const forbiddenPrimaryNavLabels = [
  "Predictions",
  "Models",
  "Signals",
  "Graph Engine",
  "Strategy Lab",
  "Feature Store",
] as const;

export const primaryNavItems: Array<{
  id: WorkspaceView;
  label: string;
  icon: typeof LayoutDashboard;
}> = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "buy", label: "Buy", icon: ShoppingCart },
  { id: "sell", label: "Sell", icon: BadgeDollarSign },
  { id: "portfolio", label: "Portfolio", icon: Shield },
  { id: "items", label: "Items", icon: FolderKanban },
  { id: "linkedItems", label: "Linked Items", icon: Link2 },
  { id: "simulations", label: "Simulations", icon: ChartLine },
  { id: "accuracy", label: "Accuracy", icon: ChartColumnBig },
  { id: "settings", label: "Settings", icon: Settings },
];
