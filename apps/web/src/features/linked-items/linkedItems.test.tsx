import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { recommendationMocks } from "../../domain/recommendation";
import { EventImpactPanel } from "./EventImpactPanel";
import { LinkedItemsView } from "./LinkedItemsView";
import { buildLinkedItemsViewModel, simpleLabelForRelation } from "./linkedItemTypes";
import { WhatIfThisMovesPanel } from "./WhatIfThisMovesPanel";

vi.mock("../../api/hooks", () => ({
  useRecommendationEvidence: () => ({
    data: null,
    isLoading: false,
    isError: false,
  }),
}));

describe("linked items view", () => {
  it("uses simple link type labels", () => {
    expect(simpleLabelForRelation("ingredient_of")).toBe("Made from");
    expect(simpleLabelForRelation("complement")).toBe("Used with");
    expect(simpleLabelForRelation("substitute")).toBe("Similar item");
    expect(simpleLabelForRelation("same_category")).toBe("Same activity");
    expect(simpleLabelForRelation("charge_conversion")).toBe("Converts into");
    expect(simpleLabelForRelation("graph_neighbor_predictive")).toBe(
      "Usually moves after",
    );
  });

  it("uses the user-facing what-if title", () => {
    const model = buildLinkedItemsViewModel(recommendationMocks.live, null)!;
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <WhatIfThisMovesPanel graphVersion={model.graphVersion} impacts={model.blastRadius} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("What happens if this moves?");
    expect(markup).toContain("Advanced subtitle: blast radius simulation");
  });

  it("renders learned-edge caveat without causal wording", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <LinkedItemsView recommendation={recommendationMocks.live} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Predictive evidence only. This does not claim one item causes the other.");
  });

  it("renders event source badges", () => {
    const markup = renderToStaticMarkup(
      <EventImpactPanel
        events={[
          {
            title: "Boss activity spike note",
            sourceType: "event",
            confidence: 0.54,
            context: "Event-linked moves are shown with source badges rather than unsupported market claims.",
          },
        ]}
      />,
    );

    expect(markup).toContain("event");
    expect(markup).toContain("Boss activity spike note");
  });
});
