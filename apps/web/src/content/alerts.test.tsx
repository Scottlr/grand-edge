import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { GlossaryProvider } from "../components/learn/GlossaryProvider";
import { LearnModal } from "../components/learn/LearnModal";
import {
  alertTemplates,
  alertTemplatesUsePlainActionLanguage,
  assertPlainAlertCopy,
} from "./alerts";
import { emptyStates, emptyStatesTeachNextAction } from "./emptyStates";

describe("alerts and empty states", () => {
  it("uses plain action language in alert templates", () => {
    expect(alertTemplatesUsePlainActionLanguage()).toBe(true);
    expect(alertTemplates.buy.title).toContain("worth buying");
    expect(alertTemplates.sell.title).toContain("cashout");
    expect(alertTemplates.wait.body).toContain("trade looks hard");
  });

  it("teaches the next action in shared empty states", () => {
    expect(emptyStatesTeachNextAction()).toBe(true);
    expect(emptyStates.noPortfolioItems.title).toBe("Track your first holding");
    expect(emptyStates.noSellRecommendations.title).toBe("No urgent sells");
    expect(emptyStates.missingAccuracy.title).toBe(
      "Past accuracy is still filling in",
    );
  });

  it("rejects forbidden technical alert phrases", () => {
    expect(() =>
      assertPlainAlertCopy("High alpha opportunity detected."),
    ).toThrow(/forbidden/i);
    expect(() =>
      assertPlainAlertCopy("Liquidity-adjusted edge degraded."),
    ).toThrow(/forbidden/i);
  });

  it("allows technical detail inside learn surfaces", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <LearnModal onOpenChange={() => undefined} open term="calibration" />
      </GlossaryProvider>,
    ).toLowerCase();

    expect(markup).toContain("advanced");
  });
});
