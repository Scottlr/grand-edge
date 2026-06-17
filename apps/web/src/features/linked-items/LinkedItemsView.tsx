import "./linkedItems.css";

import { useRecommendationEvidence } from "../../api/hooks";
import type { Recommendation } from "../../api/types";
import { TooltipTerm } from "../../components/learn/TooltipTerm";
import { DataStatePanel } from "../../components/state/DataStatePanel";
import { ActionPageHeader } from "../../views/ActionPageHeader";
import { EventImpactPanel } from "./EventImpactPanel";
import { LinkAwareOpportunityFeed } from "./LinkAwareOpportunityFeed";
import { LinkedItemPathList } from "./LinkedItemPathList";
import { PortfolioExposurePanel } from "./PortfolioExposurePanel";
import { WhatIfThisMovesPanel } from "./WhatIfThisMovesPanel";
import { buildLinkedItemsViewModel } from "./linkedItemTypes";

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

export function LinkedItemsView({
  recommendation,
}: {
  recommendation: Recommendation | null;
}) {
  const evidenceQuery = useRecommendationEvidence(
    recommendation?.recommendationId ?? null,
  );
  const model = buildLinkedItemsViewModel(recommendation, evidenceQuery.data ?? null);

  if (!recommendation || !model) {
    return (
      <section className="action-view-stack">
        <DataStatePanel
          state="empty"
          title="Linked Items"
          message="Select an item or recommendation first so Linked Items can show focused relationship paths instead of a huge graph."
          action={<button className="terminal-action-button" type="button">Return to dashboard</button>}
        />
      </section>
    );
  }

  const lowConfidenceHidden = model.paths.filter(
    (path) => path.pathConfidence < 0.45,
  ).length;

  return (
    <section className="action-view-stack">
      <ActionPageHeader
        action="WATCH CLOSELY"
        confidence={recommendation.recommendationConfidence}
        itemName={recommendation.itemName}
        why="Follow linked-item context first, then open deeper graph detail only when the relationship changes the action."
        keyNumbers={headerNumbers([
          { label: "Linked Items", value: `${model.paths.length} focused paths` },
          {
            label: "What happens if this moves?",
            value: `${model.blastRadius.length} nearby impacts`,
          },
          {
            label: "Low-confidence paths",
            value:
              lowConfidenceHidden > 0
                ? `${lowConfidenceHidden} collapsed`
                : "None collapsed",
          },
        ])}
        actions={headerActions([
          "Open item intelligence",
          "Review exposure",
          "Check event context",
        ])}
      />

      <article className="terminal-panel">
        <p className="eyebrow">
          <TooltipTerm term="linkedItem">Linked Items</TooltipTerm>
        </p>
        <p className="terminal-panel-copy">
          Default link labels stay plain: Made from, Used with, Similar item,
          Same activity, Converts into, and Usually moves after.
        </p>
        <p className="terminal-panel-copy">
          {model.usedFixtureData
            ? "Live graph payload is not available here yet, so the screen falls back to fixture examples instead of inventing hidden client-side graph logic."
            : "This screen is using stored evidence and graph-backed context from the current recommendation."}
        </p>
      </article>

      <div className="terminal-grid linked-items-grid">
        <div className="linked-items-stack">
          <LinkedItemPathList paths={model.paths} />
          <WhatIfThisMovesPanel
            graphVersion={model.graphVersion}
            impacts={model.blastRadius}
          />
        </div>
        <div className="linked-items-stack">
          <LinkAwareOpportunityFeed opportunities={model.opportunities} />
          <EventImpactPanel events={model.events} />
          <PortfolioExposurePanel exposures={model.exposures} />
        </div>
      </div>
    </section>
  );
}
