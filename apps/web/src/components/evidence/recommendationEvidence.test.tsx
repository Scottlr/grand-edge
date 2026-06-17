import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { GlossaryProvider } from "../learn/GlossaryProvider";
import { RecommendationCard } from "../cards/RecommendationCard";
import { ConfidenceMeter } from "../confidence/ConfidenceMeter";
import { getConfidenceState } from "../confidence/confidenceState";
import { EvidenceStack } from "./EvidenceStack";
import { ExpandableAdvancedPanel } from "../disclosure/ExpandableAdvancedPanel";
import { InvalidationRules } from "./InvalidationRules";
import { ModelVoteStack } from "../strategy/ModelVoteStack";
import { buildDisagreementFixture, buildRecommendationCardFixture } from "../recommendation/recommendationFixtures";

describe("recommendation evidence components", () => {
  it("maps design confidence ranges", () => {
    expect(getConfidenceState(0.2)).toBe("weak");
    expect(getConfidenceState(0.5)).toBe("uncertain");
    expect(getConfidenceState(0.68)).toBe("usable");
    expect(getConfidenceState(0.78)).toBe("strong");
    expect(getConfidenceState(0.92)).toBe("rare");
  });

  it("uses simple action labels and avoids guarantee language", () => {
    const fixture = buildRecommendationCardFixture("live");
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <RecommendationCard
          action={fixture.actionLabel}
          confidence={fixture.recommendation.confidence}
          confidenceBreakdown={fixture.recommendation.confidenceBreakdown}
          dataState={fixture.recommendation.dataState}
          expectedNetGp={fixture.recommendation.expectedNetGp}
          expectedRoi={fixture.recommendation.expectedRoi}
          horizonLabel={fixture.horizonLabel}
          invalidationRules={fixture.recommendation.invalidationRules}
          itemName={fixture.recommendation.itemName}
          learnTermIds={fixture.learnTermIds}
          modelAgreement={fixture.recommendation.modelAgreement}
          primaryReason={fixture.recommendation.primaryReason}
          reasons={fixture.recommendation.reasons}
          riskLabel={fixture.riskLabel}
          strategyVotes={fixture.recommendation.strategyVotes}
        />
      </GlossaryProvider>,
    );

    expect(markup).toContain(">BUY<");
    expect(markup.toLowerCase()).not.toContain("guaranteed");
    expect(markup.toLowerCase()).not.toContain("risk-free");
  });

  it("renders learn buttons for complex terms", () => {
    const fixture = buildRecommendationCardFixture("live");
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <EvidenceStack
          learnTermIds={fixture.learnTermIds}
          primaryReason={fixture.recommendation.primaryReason}
          reasons={fixture.recommendation.reasons}
        />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Learn: Confidence");
    expect(markup).toContain("Learn: Trade realism");
  });

  it("hides advanced model fields by default", () => {
    const markup = renderToStaticMarkup(
      <ExpandableAdvancedPanel title="Advanced recommendation detail">
        <p>Hidden detail</p>
      </ExpandableAdvancedPanel>,
    );

    expect(markup).toContain("<details");
    expect(markup).not.toContain("<details open");
  });

  it("renders missing accuracy honestly", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <ConfidenceMeter confidence={0.61} dataQualityLabel="stale" modelAgreementLabel="mixed agreement" recentAccuracy={null} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Not enough history yet.");
  });

  it("shows disagreement rows clearly", () => {
    const fixture = buildDisagreementFixture();
    const markup = renderToStaticMarkup(<ModelVoteStack votes={fixture.recommendation.strategyVotes} />);

    expect(markup).toContain("spread_edge_v1");
    expect(markup).toContain("mean_reversion_v1");
    expect(markup).toContain("WATCH");
  });

  it("renders structured invalidation thresholds", () => {
    const fixture = buildRecommendationCardFixture("live");
    const markup = renderToStaticMarkup(<InvalidationRules rules={fixture.recommendation.invalidationRules} />);

    expect(markup).toContain("final_score");
    expect(markup).toContain("&lt;");
    expect(markup).toContain("0.05");
  });

  it("handles degraded recommendation evidence", () => {
    const fixture = buildRecommendationCardFixture("degraded");
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <RecommendationCard
          action={fixture.actionLabel}
          confidence={fixture.recommendation.confidence}
          confidenceBreakdown={fixture.recommendation.confidenceBreakdown}
          dataState={fixture.recommendation.dataState}
          expectedNetGp={fixture.recommendation.expectedNetGp}
          expectedRoi={fixture.recommendation.expectedRoi}
          horizonLabel={fixture.horizonLabel}
          invalidationRules={fixture.recommendation.invalidationRules}
          itemName={fixture.recommendation.itemName}
          learnTermIds={fixture.learnTermIds}
          modelAgreement={fixture.recommendation.modelAgreement}
          primaryReason={fixture.recommendation.primaryReason}
          reasons={fixture.recommendation.reasons}
          riskLabel={fixture.riskLabel}
          strategyVotes={fixture.recommendation.strategyVotes}
        />
      </GlossaryProvider>,
    );

    expect(markup).toContain("degraded");
  });
});
