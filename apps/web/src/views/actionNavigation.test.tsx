import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { GlossaryProvider } from "../components/learn/GlossaryProvider";
import { recommendationMocks } from "../domain/recommendation";
import { forbiddenPrimaryNavLabels, primaryNavItems } from "../navigation/routes";
import {
  AccuracyView,
  BuyView,
  DashboardView,
  SellView,
} from "./ActionJourneyViews";
import { simpleActionLabel } from "../components/recommendation/recommendationFixtures";

describe("action-centered navigation", () => {
  it("uses the required primary nav labels", () => {
    expect(primaryNavItems.map((item) => item.label)).toEqual([
      "Dashboard",
      "Buy",
      "Sell",
      "Portfolio",
      "Items",
      "Linked Items",
      "Simulations",
      "Accuracy",
      "Settings",
    ]);
  });

  it("rejects technical nav labels", () => {
    const labels = primaryNavItems.map((item) => item.label);

    forbiddenPrimaryNavLabels.forEach((label) => {
      expect(labels).not.toContain(label);
    });
  });

  it("maps avoid and watch states to beginner-safe action labels", () => {
    expect(simpleActionLabel(recommendationMocks.live)).toBe("BUY");
    expect(simpleActionLabel(recommendationMocks.degraded)).toBe("BUY");
    expect(
      simpleActionLabel({
        ...recommendationMocks.live,
        action: "avoid",
        executionConfidence: 0.61,
      }),
    ).toBe("DO NOT ADD");
    expect(
      simpleActionLabel({
        ...recommendationMocks.live,
        action: "watch",
      }),
    ).toBe("WATCH CLOSELY");
  });

  it("starts the dashboard with buy and sell action cards", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <DashboardView positions={[]} recommendations={[recommendationMocks.live]} />
      </GlossaryProvider>,
    );

    expect(markup.indexOf("BUY Abyssal whip")).toBeLessThan(markup.indexOf("What changed"));
    expect(markup).toContain("Items to avoid or watch");
  });

  it("buy and sell pages use beginner-first columns", () => {
    const buyMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <BuyView recommendation={recommendationMocks.live} />
      </GlossaryProvider>,
    );
    const sellMarkup = renderToStaticMarkup(
      <GlossaryProvider>
        <SellView recommendation={{ ...recommendationMocks.live, action: "cashout" }} />
      </GlossaryProvider>,
    );

    expect(buyMarkup).toContain("Expected profit");
    expect(buyMarkup).toContain("Suggested quantity");
    expect(buyMarkup).toContain("Timeframe");
    expect(sellMarkup).toContain("Your profit");
    expect(sellMarkup).toContain("Suggested sell price");
    expect(sellMarkup).toContain("Confidence");
  });

  it("accuracy page starts with trust language instead of model jargon", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <AccuracyView recommendation={recommendationMocks.live} />
      </GlossaryProvider>,
    );

    expect(markup).toContain("Can I trust this?");
    expect(markup).not.toContain("Strategy Lab");
  });
});
