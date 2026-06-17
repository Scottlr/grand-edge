import { formatStrategyGp, formatStrategyPercent, formatStrategyWeight, type StrategyLabRow } from "../../domain/strategy";
import { StrategyToggle } from "./StrategyToggle";

function statusLabel(status: StrategyLabRow["status"]) {
  switch (status) {
    case "active":
      return "Active";
    case "disabled":
      return "Disabled";
    case "degraded":
      return "Degraded";
    case "insufficient_data":
      return "Not enough data yet";
    case "error":
    default:
      return "Unavailable";
  }
}

export function StrategyTable({
  rows,
  selectedStrategyId,
  pendingStrategyId,
  onSelect,
  onToggle,
}: {
  rows: StrategyLabRow[];
  selectedStrategyId: string | null;
  pendingStrategyId: string | null;
  onSelect: (strategyId: string) => void;
  onToggle: (strategyId: string, enabled: boolean) => void;
}) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Strategy table</p>
      <div className="command-center-table-wrap">
        <table className="command-center-table strategy-lab-table">
          <thead>
            <tr>
              <th>Strategy</th>
              <th>Enabled</th>
              <th>Weight</th>
              <th>30d net GP</th>
              <th>30d accuracy</th>
              <th>Current confidence</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => {
              const pending = pendingStrategyId === row.strategyId;
              const selected = selectedStrategyId === row.strategyId;

              return (
                <tr
                  aria-selected={selected}
                  className={selected ? "command-center-row-selected" : undefined}
                  key={row.strategyId}
                  onClick={() => onSelect(row.strategyId)}
                >
                  <td>
                    <strong>{row.displayName}</strong>
                    <p className="terminal-panel-copy">{row.disabledReason ?? "Method detail is available in the panel."}</p>
                  </td>
                  <td>
                    <StrategyToggle
                      checked={row.enabled}
                      disabled={!row.canToggle}
                      label={`Toggle ${row.displayName}`}
                      onChange={(next) => onToggle(row.strategyId, next)}
                      pending={pending}
                    />
                  </td>
                  <td>{formatStrategyWeight(row.weight)}</td>
                  <td>{formatStrategyGp(row.netGp30d)}</td>
                  <td>{formatStrategyPercent(row.accuracy30d)}</td>
                  <td>{formatStrategyPercent(row.currentConfidence)}</td>
                  <td>{statusLabel(row.status)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </article>
  );
}
