import { AlertTriangle, ArrowDownToLine, ArrowUpToLine, CircleOff, Gauge, Layers3, ShieldAlert } from "lucide-react";

import type {
  Position,
  Recommendation,
  RecommendationAction,
  SimulationRun,
  StrategyStatus,
} from "../api/types";

type TerminalGridProps = {
  activeViewLabel: string;
  itemsById: Map<number, { iconUrl: string | null; name: string }>;
  positions: Position[];
  recommendations: Recommendation[];
  simulations: SimulationRun[];
  strategies: StrategyStatus[];
  strategyMutationPendingId: string | null;
  onSelectRecommendation: (recommendationId: string, itemId: number) => void;
  onToggleStrategy: (strategyId: string, enabled: boolean) => void;
};

type CommandCard = {
  id: string;
  title: string;
  summary: string;
  recommendation: Recommendation | null;
  tone: RecommendationAction | "hold";
  icon: typeof Gauge;
};

function recommendationTone(action: RecommendationAction) {
  switch (action) {
    case "buy":
    case "add":
      return "buy";
    case "cashout":
      return "sell";
    case "watch":
      return "wait";
    case "avoid":
      return "avoid";
    case "hold":
    default:
      return "hold";
  }
}

function formatNumber(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }

  return new Intl.NumberFormat("en-GB").format(value);
}

function buildCommandCards(
  recommendations: Recommendation[],
  positions: Position[],
  itemsById: Map<number, { iconUrl: string | null; name: string }>,
): CommandCard[] {
  const buyCandidates = recommendations.filter((entry) => entry.action === "buy" || entry.action === "add");
  const cashoutCandidates = recommendations.filter((entry) => entry.action === "cashout");
  const watchCandidates = recommendations.filter((entry) => entry.action === "watch");
  const disagreementCandidates = recommendations.filter((entry) => {
    const voteSides = new Set(entry.strategyVotes.map((vote) => vote.side));
    return voteSides.size > 1;
  });
  const atRiskCandidates = recommendations.filter(
    (entry) =>
      positions.some((position) => position.itemId === entry.itemId) &&
      (entry.action === "avoid" || entry.action === "cashout"),
  );

  const cards: CommandCard[] = [
    {
      id: "top-buy",
      title: "Top buy",
      summary: "Highest current recommendation score among BUY and ADD actions.",
      recommendation: buyCandidates.sort((left, right) => right.score - left.score)[0] ?? null,
      tone: "buy",
      icon: ArrowUpToLine,
    },
    {
      id: "top-cashout",
      title: "Top cashout",
      summary: "Strongest CASHOUT call for existing winners or de-risking exits.",
      recommendation: cashoutCandidates.sort((left, right) => right.score - left.score)[0] ?? null,
      tone: "cashout",
      icon: ArrowDownToLine,
    },
    {
      id: "best-risk-adjusted",
      title: "Best risk-adjusted",
      summary: "Highest expected net GP weighted by recommendation confidence.",
      recommendation:
        recommendations
          .filter((entry) => entry.expectedNetGp !== null)
          .sort(
            (left, right) =>
              (right.expectedNetGp ?? 0) * right.recommendationConfidence -
              (left.expectedNetGp ?? 0) * left.recommendationConfidence,
          )[0] ?? null,
      tone: "hold",
      icon: Gauge,
    },
    {
      id: "high-spread-low-confidence",
      title: "High spread / low confidence",
      summary: "Placeholder queue until spread-focused trust DTOs arrive from the API.",
      recommendation:
        watchCandidates.sort((left, right) => left.recommendationConfidence - right.recommendationConfidence)[0] ?? null,
      tone: "watch",
      icon: CircleOff,
    },
    {
      id: "model-disagreement",
      title: "Model disagreement",
      summary: "Recommendations whose strategy votes pull in different directions.",
      recommendation:
        disagreementCandidates.sort(
          (left, right) => right.strategyVotes.length - left.strategyVotes.length,
        )[0] ?? null,
      tone: "watch",
      icon: Layers3,
    },
    {
      id: "holdings-at-risk",
      title: "Holdings at risk",
      summary: "Current positions that overlap with AVOID or CASHOUT recommendations.",
      recommendation: atRiskCandidates[0] ?? null,
      tone: "avoid",
      icon: ShieldAlert,
    },
  ];

  return cards.map((card) => {
    if (!card.recommendation) {
      return card;
    }

    const item = itemsById.get(card.recommendation.itemId);
    return {
      ...card,
      summary: item ? `${item.name} - ${card.summary}` : card.summary,
    };
  });
}

export function TerminalGrid({
  activeViewLabel,
  itemsById,
  onSelectRecommendation,
  onToggleStrategy,
  positions,
  recommendations,
  simulations,
  strategies,
  strategyMutationPendingId,
}: TerminalGridProps) {
  const commandCards = buildCommandCards(recommendations, positions, itemsById);

  return (
    <section className="terminal-grid">
      <div className="terminal-primary">
        <article className="terminal-panel terminal-hero-panel">
          <div className="terminal-panel-header">
            <div>
              <p className="eyebrow">Active workspace</p>
              <h2>{activeViewLabel}</h2>
            </div>
            <p className="terminal-panel-copy">
              Command-center tiles stay backend-led. This shell derives ordering from existing recommendation
              fields and marks unavailable views honestly until later trust and explainer contracts land.
            </p>
          </div>

          <div className="terminal-card-grid">
            {commandCards.map((card) => {
              const Icon = card.icon;
              const recommendation = card.recommendation;
              const tone = recommendation ? recommendationTone(recommendation.action) : recommendationTone(card.tone);

              return (
                <article className="terminal-command-card" key={card.id}>
                  <div className="terminal-command-head">
                    <span className={`terminal-tone terminal-tone-${tone}`}>
                      <Icon size={16} />
                      {card.title}
                    </span>
                    <span className="terminal-mono">
                      {recommendation ? `${Math.round(recommendation.recommendationConfidence * 100)}% confidence` : "Awaiting data"}
                    </span>
                  </div>
                  <h3>{recommendation ? itemsById.get(recommendation.itemId)?.name ?? "Unknown item" : "No matching recommendation"}</h3>
                  <p>{card.summary}</p>
                  <div className="terminal-command-metrics">
                    <div>
                      <span className="eyebrow">Net GP</span>
                      <strong>{recommendation ? formatNumber(recommendation.expectedNetGp) : "Unavailable"}</strong>
                    </div>
                    <div>
                      <span className="eyebrow">Action</span>
                      <strong>{recommendation ? recommendation.action.toUpperCase() : "NONE"}</strong>
                    </div>
                  </div>
                  <button
                    className="terminal-action-button"
                    disabled={!recommendation}
                    onClick={() => {
                      if (recommendation) {
                        onSelectRecommendation(recommendation.recommendationId, recommendation.itemId);
                      }
                    }}
                    type="button"
                  >
                    {recommendation ? "Inspect recommendation" : "No recommendation available"}
                  </button>
                </article>
              );
            })}
          </div>
        </article>

        <article className="terminal-panel">
          <div className="terminal-panel-header-inline">
            <div>
              <p className="eyebrow">Strategy laboratory</p>
              <h3>Enable and disable strategy surfaces</h3>
            </div>
            <p className="terminal-panel-copy">
              PATCH-backed toggles invalidate the strategy cache and respect backend authority.
            </p>
          </div>
          <div className="terminal-strategy-list">
            {strategies.map((strategy) => (
              <label className="terminal-strategy-row" key={strategy.strategyId}>
                <div>
                  <span className="terminal-strategy-name">{strategy.strategyId}</span>
                  <span className="terminal-strategy-meta">
                    {strategy.enabled ? "Enabled for recommendation runs" : "Disabled"}
                  </span>
                </div>
                <input
                  checked={strategy.enabled}
                  disabled={strategyMutationPendingId === strategy.strategyId}
                  onChange={() => onToggleStrategy(strategy.strategyId, !strategy.enabled)}
                  type="checkbox"
                />
              </label>
            ))}
          </div>
        </article>
      </div>

      <div className="terminal-secondary">
        <article className="terminal-panel">
          <div className="terminal-panel-header-inline">
            <div>
              <p className="eyebrow">Positions</p>
              <h3>Backend-held positions</h3>
            </div>
          </div>
          <div className="terminal-list">
            {positions.length > 0 ? (
              positions.map((position) => (
                <div className="terminal-list-row" key={position.positionId}>
                  <div>
                    <strong>{itemsById.get(position.itemId)?.name ?? `Item ${position.itemId}`}</strong>
                    <p>{position.quantity} units - avg buy {formatNumber(position.avgBuyPrice)} gp</p>
                  </div>
                  <span className="terminal-mono">
                    {position.boughtAt ? new Date(position.boughtAt).toLocaleDateString("en-GB") : "Undated"}
                  </span>
                </div>
              ))
            ) : (
              <div className="terminal-empty-state">
                <AlertTriangle size={18} />
                No positions returned by the API yet.
              </div>
            )}
          </div>
        </article>

        <article className="terminal-panel">
          <div className="terminal-panel-header-inline">
            <div>
              <p className="eyebrow">Simulation runs</p>
              <h3>Paper-trading activity</h3>
            </div>
          </div>
          <div className="terminal-list">
            {simulations.length > 0 ? (
              simulations.map((run) => (
                <div className="terminal-list-row" key={run.runId}>
                  <div>
                    <strong>{run.name}</strong>
                    <p>{run.status}</p>
                  </div>
                  <span className="terminal-mono">
                    {new Date(run.startedAt).toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" })}
                  </span>
                </div>
              ))
            ) : (
              <div className="terminal-empty-state">
                <AlertTriangle size={18} />
                No simulation runs are available yet.
              </div>
            )}
          </div>
        </article>
      </div>
    </section>
  );
}
