import type { WorkspaceView } from "../state/workspaceStore";
import type { Item } from "../api/types";
import { primaryNavItems } from "../navigation/routes";
import { ChevronLeft, ChevronRight } from "lucide-react";

type SidebarProps = {
  activeView: WorkspaceView;
  collapsed: boolean;
  items: Item[];
  selectedItemId: number | null;
  onSelectView: (view: WorkspaceView) => void;
  onSelectItem: (itemId: number) => void;
  onToggleCollapsed: () => void;
};

export function Sidebar({
  activeView,
  collapsed,
  items,
  selectedItemId,
  onSelectItem,
  onSelectView,
  onToggleCollapsed,
}: SidebarProps) {
  return (
    <aside className={`terminal-sidebar ${collapsed ? "terminal-sidebar-collapsed" : ""}`}>
      <div className="terminal-sidebar-header">
        <div>
          <p className="eyebrow">Workspace</p>
          {!collapsed ? <p className="terminal-sidebar-title">Terminal shell</p> : null}
        </div>
        <button className="terminal-icon-button" onClick={onToggleCollapsed} type="button">
          {collapsed ? <ChevronRight size={18} /> : <ChevronLeft size={18} />}
        </button>
      </div>

      <nav className="terminal-sidebar-nav" aria-label="workspace views">
        {primaryNavItems.map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              className={`terminal-nav-button ${
                activeView === item.id ? "terminal-nav-button-active" : ""
              }`}
              onClick={() => onSelectView(item.id)}
              type="button"
            >
              <Icon size={18} />
              {!collapsed ? <span>{item.label}</span> : null}
            </button>
          );
        })}
      </nav>

      <div className="terminal-sidebar-section">
        {!collapsed ? <p className="eyebrow">Watchlist</p> : null}
        <div className="terminal-watchlist">
          {items.slice(0, collapsed ? 4 : 7).map((item) => (
            <button
              key={item.itemId}
              className={`terminal-watch-item ${
                selectedItemId === item.itemId ? "terminal-watch-item-active" : ""
              }`}
              onClick={() => onSelectItem(item.itemId)}
              type="button"
              title={item.name}
            >
              {!collapsed ? (
                <>
                  {item.icon ? (
                    <img alt={item.name} className="terminal-watch-icon" src={item.icon.cdnUrl} />
                  ) : (
                    <span className="terminal-watch-fallback">{item.name.slice(0, 2).toUpperCase()}</span>
                  )}
                  <span className="terminal-watch-name">{item.name}</span>
                </>
              ) : item.icon ? (
                <img alt={item.name} className="terminal-watch-icon" src={item.icon.cdnUrl} />
              ) : (
                <span className="terminal-watch-fallback">{item.name.slice(0, 2).toUpperCase()}</span>
              )}
            </button>
          ))}
        </div>
      </div>
    </aside>
  );
}
