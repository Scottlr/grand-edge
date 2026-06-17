import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { GlossaryProvider } from "../components/learn/GlossaryProvider";
import { LearnModal } from "../components/learn/LearnModal";
import { TooltipTerm } from "../components/learn/TooltipTerm";
import { assertDefaultUiCopy } from "./copyRules";
import { getGlossaryEntry, glossaryContainsAllRequiredTerms } from "./glossary";

describe("glossary content", () => {
  it("contains all required terms with quick, learn, and advanced content", () => {
    expect(glossaryContainsAllRequiredTerms()).toBe(true);
  });

  it("keeps required high-signal terms available", () => {
    expect(getGlossaryEntry("confidence").label).toBe("Confidence");
    expect(getGlossaryEntry("spread").label).toBe("Spread");
    expect(getGlossaryEntry("executionConfidence").label).toBe("Trade realism");
    expect(getGlossaryEntry("linkedItem").label).toBe("Linked item");
    expect(getGlossaryEntry("simulation").label).toBe("Simulation");
    expect(getGlossaryEntry("buyLimit").label).toBe("Buy limit");
    expect(getGlossaryEntry("calibration").label).toBe("Confidence honesty");
  });

  it("gives tooltip terms an accessible name", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <TooltipTerm term="executionConfidence" />
      </GlossaryProvider>,
    );

    expect(markup).toContain(">Trade realism<");
  });

  it("renders learn modal quick, example, and advanced sections", () => {
    const markup = renderToStaticMarkup(
      <GlossaryProvider>
        <LearnModal onOpenChange={() => undefined} open term="spread" />
      </GlossaryProvider>,
    );

    expect(markup).toContain("The gap between the buy-side and sell-side price.");
    expect(markup).toContain("If an item buys for 1,000 GP and sells for 950 GP, the spread is 50 GP.");
    expect(markup).toContain("Advanced");
    expect(markup).toContain("spread_pct = (high - low) / midpoint");
  });

  it("rejects forbidden default UI jargon", () => {
    expect(() => assertDefaultUiCopy("This screen shows calibration and graph propagation.")).toThrow(
      /forbidden technical terms/i,
    );
  });
});
