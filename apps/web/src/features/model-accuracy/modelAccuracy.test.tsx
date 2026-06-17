import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { GlossaryProvider } from "../../components/learn/GlossaryProvider";
import { recommendationMocks } from "../../domain/recommendation";
import { AccuracyStatus } from "./AccuracyStatus";
import { CalibrationPanel } from "./CalibrationPanel";
import { MetricSummaryGrid } from "./MetricSummaryGrid";
import { modelAccuracyFixtures, trustSummaryFixture } from "./modelAccuracyFixtures";
import { ModelAccuracyView } from "./ModelAccuracyView";
import { selectAccuracyModel } from "./modelAccuracySelectors";
import { TrustSummary } from "./TrustSummary";

describe("model accuracy view", () => {
  it("uses trust language in the summary section", () => {
    const markup = renderToStaticMarkup(<TrustSummary summary={trustSummaryFixture} />);

    expect(markup).toContain("Can I trust this?");
    expect(markup).toContain("BUY calls profitable");
  });

  it("renders unavailable for null metrics", () => {
    const degradedModel = modelAccuracyFixtures.find((model) => model.strategyId === "volatility_filter_v1");
    const markup = renderToStaticMarkup(<MetricSummaryGrid model={degradedModel!} />);

    expect(markup).toContain("Unavailable");
  });

  it("shows sample size in the calibration panel", () => {
    const markup = renderToStaticMarkup(<CalibrationPanel model={modelAccuracyFixtures[0]} />);

    expect(markup).toContain("Recent sample size: 24");
  });

  it("marks an overconfident model clearly", () => {
    const overconfidentModel = modelAccuracyFixtures.find((model) => model.strategyId === "momentum_v1");
    const markup = renderToStaticMarkup(<AccuracyStatus model={overconfidentModel!} />);

    expect(markup).toContain("too certain");
  });

  it("keeps Brier score hidden until advanced detail is opened", () => {
    const closedMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <ModelAccuracyView recommendation={recommendationMocks.live} />
      </GlossaryProvider>,
    );
    const openMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <ModelAccuracyView initialAdvancedOpen recommendation={recommendationMocks.live} />
      </GlossaryProvider>,
    );

    expect(closedMarkup).not.toContain("Brier score");
    expect(openMarkup).toContain("Brier score");
  });

  it("filters by strategy and window when choosing the displayed model", () => {
    const monthModel = selectAccuracyModel(modelAccuracyFixtures, "spread_edge_v1", "30d");
    const lowSampleModel = selectAccuracyModel(modelAccuracyFixtures, "mean_reversion_v1", "7d");

    expect(monthModel?.sampleSize).toBe(81);
    expect(lowSampleModel?.sampleSize).toBe(6);
  });
});
