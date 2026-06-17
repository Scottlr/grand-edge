import { useMemo, useState, type FormEvent } from "react";

import {
  useCreatePosition,
  useRiskProfile,
  useUpdatePosition,
} from "../../api/hooks";
import type { Position, Recommendation } from "../../api/types";
import { ActionPageHeader } from "../../views/ActionPageHeader";
import { DataStatePanel } from "../../components/state/DataStatePanel";
import {
  riskPreferenceFromProfile,
  toPositionRequest,
  type PositionFormValues,
} from "../../domain/portfolio";
import {
  buildHoldingGuidance,
  buildPortfolioSummary,
  firstPortfolioRecommendation,
  recommendationToHoldingAction,
} from "./portfolioFixtures";
import { emptyStates } from "../../content/emptyStates";
import { HoldingActionCard } from "./HoldingActionCard";
import { PortfolioSummary } from "./PortfolioSummary";
import { PositionForm } from "./PositionForm";

function initialForm(
  recommendation: Recommendation | null,
  positions: Position[],
  riskPreference: PositionFormValues["riskPreference"],
): PositionFormValues {
  const first = positions[0];
  if (first) {
    return {
      itemId: first.itemId,
      quantity: first.quantity,
      avgBuyPrice: first.avgBuyPrice,
      boughtAt: first.boughtAt ?? undefined,
      notes: first.notes ?? undefined,
      riskPreference,
      targetProfitGp: recommendation?.expectedNetGp ?? undefined,
    };
  }

  return {
    itemId: recommendation?.itemId ?? 0,
    quantity: recommendation?.strategyVotes[0]?.maxQuantity ?? 1,
    avgBuyPrice: recommendation?.strategyVotes[0]?.targetEntry ?? 0,
    boughtAt: recommendation?.asOf ?? undefined,
    notes: "",
    riskPreference,
    targetProfitGp: recommendation?.expectedNetGp ?? undefined,
  };
}

function formatGp(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unavailable";
  }
  return `${value} gp`;
}

export function PortfolioView({
  positions,
  recommendations,
}: {
  positions: Position[];
  recommendations: Recommendation[];
}) {
  const selectedRecommendation = useMemo(
    () => firstPortfolioRecommendation(recommendations, positions),
    [positions, recommendations],
  );
  const riskProfile = useRiskProfile();
  const riskPreference = riskPreferenceFromProfile(riskProfile.data);
  const [form, setForm] = useState<PositionFormValues>(() =>
    initialForm(selectedRecommendation, positions, riskPreference),
  );
  const createPosition = useCreatePosition();
  const updatePosition = useUpdatePosition();

  const guidance = useMemo(
    () => buildHoldingGuidance(positions, recommendations),
    [positions, recommendations],
  );
  const summary = useMemo(() => buildPortfolioSummary(guidance), [guidance]);
  const action = selectedRecommendation
    ? recommendationToHoldingAction(selectedRecommendation, positions.length > 0)
    : positions.length > 0
      ? "HOLD"
      : "WATCH CLOSELY";

  function submitPosition(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (positions[0]) {
      void updatePosition.mutate({
        id: positions[0].positionId,
        body: toPositionRequest(form),
      });
      return;
    }
    void createPosition.mutate(toPositionRequest(form));
  }

  return (
    <section className="detailed-view-stack">
      <ActionPageHeader
        action={action}
        confidence={selectedRecommendation?.recommendationConfidence ?? null}
        itemName={selectedRecommendation?.itemName}
        why={
          selectedRecommendation?.primaryReason ??
          "Start with the hold-or-sell call for items you already own, then check the numbers behind it."
        }
        keyNumbers={
          <>
            <div className="action-keynumber">
              <span className="eyebrow">Tracked items</span>
              <strong>{positions.length}</strong>
            </div>
            <div className="action-keynumber">
              <span className="eyebrow">Suggested cashout</span>
              <strong>{formatGp(selectedRecommendation?.strategyVotes[0]?.targetExit)}</strong>
            </div>
            <div className="action-keynumber">
              <span className="eyebrow">Profit after tax</span>
              <strong>{formatGp(summary.estimatedProfitAfterTax)}</strong>
            </div>
          </>
        }
        actions={
          <>
            <button className="terminal-action-button" type="button">
              Show why
            </button>
            <button className="terminal-action-button" type="button">
              Review holdings
            </button>
          </>
        }
      />

      {positions.length === 0 ? (
        <DataStatePanel
          state="empty"
          title={emptyStates.noPortfolioItems.title}
          message={emptyStates.noPortfolioItems.message}
        />
      ) : (
        <>
          <PortfolioSummary summary={summary} />
          <section className="portfolio-holding-grid">
            {guidance.map((entry) => (
              <HoldingActionCard guidance={entry} key={entry.itemId} />
            ))}
          </section>
        </>
      )}

      <PositionForm
        onSubmit={submitPosition}
        onChange={setForm}
        submitLabel={positions.length > 0 ? "Update holding" : "Track holding"}
        values={form}
      />
    </section>
  );
}
