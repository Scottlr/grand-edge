import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { SimulationRun, StrategyStatus } from "../../api/types";
import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { buildDisagreementFixture } from "../../components/recommendation/recommendationFixtures";
import { recommendationMocks } from "../../domain/recommendation";
import { CommandCenterView } from "./CommandCenterView";
import { OpportunityTable } from "./OpportunityTable";
import { RecommendationInspector } from "./RecommendationInspector";

const EMPTY_SIMULATIONS: SimulationRun[] = [];
const STRATEGIES: StrategyStatus[] = [
  { strategyId: "spread_edge_v1", enabled: true },
  { strategyId: "mean_reversion_v1", enabled: true },
];

describe("command center", () => {
  it("starts with buy and sell action cards before later sections", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <CommandCenterView
          onSelectRecommendation={() => undefined}
          positions={[]}
          recommendations={[recommendationMocks.live]}
          selectedRecommendationId={recommendationMocks.live.recommendationId}
          simulations={EMPTY_SIMULATIONS}
          strategies={STRATEGIES}
        />
      </GlossaryProvider>,
    );

    expect(markup.indexOf("Best thing to buy")).toBeLessThan(markup.indexOf("Opportunity table"));
    expect(markup.indexOf("Best thing to sell")).toBeLessThan(markup.indexOf("Model health"));
  });

  it("uses simple action labels in the opportunity table", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <OpportunityTable
          onSelectRecommendation={() => undefined}
          recommendations={[recommendationMocks.live]}
          selectedRecommendationId={recommendationMocks.live.recommendationId}
        />
      </GlossaryProvider>,
    );

    expect(markup).toContain(">BUY<");
  });

  it("renders item icons from backend-provided CDN urls", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <OpportunityTable
          onSelectRecommendation={() => undefined}
          recommendations={[
            {
              ...recommendationMocks.live,
              itemName: "Chef's hat",
              itemIcon: {
                sourceFileName: "Chef's hat.png",
                canonicalFileName: "Chef%27s_hat.png",
                cdnUrl: "https://oldschool.runescape.wiki/images/Chef%27s_hat.png",
                source: "mapping_icon",
              },
            },
          ]}
          selectedRecommendationId={recommendationMocks.live.recommendationId}
        />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Chef%27s_hat.png");
  });

  it("keeps technical forbidden copy out of default command-center text", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <CommandCenterView
          onSelectRecommendation={() => undefined}
          positions={[]}
          recommendations={[recommendationMocks.live]}
          selectedRecommendationId={recommendationMocks.live.recommendationId}
          simulations={EMPTY_SIMULATIONS}
          strategies={STRATEGIES}
        />
      </GlossaryProvider>,
    ).toLowerCase();

    expect(markup).not.toContain("feature store");
    expect(markup).not.toContain("graph engine");
    expect(markup).not.toContain("execution model");
  });

  it("renders separate prediction, execution, and recommendation confidence values", () => {
    const disagreement = buildDisagreementFixture().recommendation;
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <RecommendationInspector recommendation={disagreement} simulations={EMPTY_SIMULATIONS} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Prediction confidence");
    expect(markup).toContain("Trade realism");
    expect(markup).toContain("Recommendation confidence");
  });

  it("frames weak execution as watch-first, not a confident buy", () => {
    const disagreement = buildDisagreementFixture().recommendation;
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <RecommendationInspector recommendation={disagreement} simulations={EMPTY_SIMULATIONS} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("watch-first territory");
    expect(markup).toContain(">WATCH CLOSELY<");
  });

  it("shows stale and empty states honestly", () => {
    const staleMarkup = renderToStaticMarkup(
      <OpportunityTable
        onSelectRecommendation={() => undefined}
        recommendations={[recommendationMocks.stale]}
        selectedRecommendationId={null}
      />,
    );
    const emptyMarkup = renderToStaticMarkup(
      <OpportunityTable
        onSelectRecommendation={() => undefined}
        recommendations={[]}
        selectedRecommendationId={null}
      />,
    );

    expect(staleMarkup).toContain("Data is stale. Recommendations are paused until fresh prices arrive.");
    expect(emptyMarkup).toContain("No strong buys right now.");
  });

  it("marks the selected recommendation row and keeps inspector content aligned", () => {
    const alternate = {
      ...recommendationMocks.live,
      recommendationId: "rec-two",
      itemId: 11840,
      itemName: "Dragon boots",
      primaryReason: "Second opportunity for selection coverage.",
    };

    const tableMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <OpportunityTable
          onSelectRecommendation={() => undefined}
          recommendations={[recommendationMocks.live, alternate]}
          selectedRecommendationId="rec-two"
        />
      </GlossaryProvider>,
    );
    const inspectorMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <RecommendationInspector recommendation={alternate} simulations={EMPTY_SIMULATIONS} />
      </GlossaryProvider>,
    );

    expect(tableMarkup).toContain('aria-selected="true"');
    expect(inspectorMarkup).toContain("Dragon boots");
  });
});
