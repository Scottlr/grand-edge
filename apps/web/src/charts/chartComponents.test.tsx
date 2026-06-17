import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { CalibrationBucketGraph } from "./CalibrationBucketGraph";
import { DrawdownGraph } from "./DrawdownGraph";
import { ForecastBandGraph } from "./ForecastBandGraph";
import { OverlayControls } from "./OverlayControls";
import { PricePathGraph } from "./PricePathGraph";
import { chartFixtureCalibrationBuckets, chartFixtureDrawdown, chartFixtureForecastBand, chartFixtureMarkers, chartFixturePricePoints } from "./chartFixtures";
import { technicalChartTermsHiddenByDefault } from "./chartTypes";
import { buildForecastBand, intervalPricesToTimePoints, timePointsToPricePoints } from "./scales";

describe("chart components", () => {
  it("skips missing mid and high values instead of substituting zeroes", () => {
    const points = intervalPricesToTimePoints([
      {
        itemId: 4151,
        bucketStart: "2026-06-16T10:00:00Z",
        interval: "1h",
        avgHighPrice: 100200,
        highPriceVolume: 140,
        avgLowPrice: 99400,
        lowPriceVolume: 122,
      },
      {
        itemId: 4151,
        bucketStart: "2026-06-16T11:00:00Z",
        interval: "1h",
        avgHighPrice: null,
        highPriceVolume: 120,
        avgLowPrice: 99700,
        lowPriceVolume: 116,
      },
    ]);

    expect(points[1]?.mid).toBe(99700);
    expect(points[1]?.high).toBeNull();
  });

  it("does not render a fake likely price range when bounds are unavailable", () => {
    const markup = renderToStaticMarkup(
      <ForecastBandGraph
        points={[
          { timestamp: "2026-06-16T10:00:00Z", label: "10:00", lower: null, predicted: 99800, upper: null },
        ]}
      />,
    );

    expect(markup).toContain("No likely price range is available yet.");
  });

  it("uses plain default labels and hides technical jargon by default", () => {
    const markup = renderToStaticMarkup(<OverlayControls />);

    expect(markup).toContain("Show advanced chart layers");
    technicalChartTermsHiddenByDefault.forEach((term) => {
      expect(markup.toLowerCase()).not.toContain(term);
    });
  });

  it("renders a nonblank current price path with markers", () => {
    const markup = renderToStaticMarkup(
      <PricePathGraph forecastBand={buildForecastBand(chartFixturePricePoints)} markers={chartFixtureMarkers} points={chartFixturePricePoints} />,
    );

    expect(markup).toContain("Current price path graph");
    expect(markup).toContain("mini-chart-marker-entry");
  });

  it("renders calibration buckets with sample size context", () => {
    const markup = renderToStaticMarkup(<CalibrationBucketGraph buckets={chartFixtureCalibrationBuckets} />);

    expect(markup).toContain("Confidence honesty graph");
    expect(markup).toContain("n=21");
  });

  it("renders worst temporary drop states without hiding skipped replays", () => {
    const markup = renderToStaticMarkup(<DrawdownGraph points={chartFixtureDrawdown} />);

    expect(markup).toContain("Worst temporary drop graph");
    expect(markup).toContain("bar-chart-bar-skipped");
  });

  it("exports reusable fixtures that stay renderable", () => {
    const markup = renderToStaticMarkup(<ForecastBandGraph points={chartFixtureForecastBand} />);
    const pricePoints = timePointsToPricePoints(intervalPricesToTimePoints([
      {
        itemId: 4151,
        bucketStart: "2026-06-16T10:00:00Z",
        interval: "1h",
        avgHighPrice: 100200,
        highPriceVolume: 140,
        avgLowPrice: 99400,
        lowPriceVolume: 122,
      },
    ]));

    expect(markup).toContain("Likely price range graph");
    expect(pricePoints[0]?.mid).toBe(99800);
  });
});
