import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { DataState } from "../../domain/recommendation";
import { DataStatePanel } from "./DataStatePanel";

describe("DataStatePanel", () => {
  it("renders all supported states", () => {
    const states: DataState[] = [
      "loading",
      "live",
      "stale",
      "degraded",
      "empty",
      "error",
    ];

    const markup = states
      .map((state) => renderToStaticMarkup(<DataStatePanel state={state} />))
      .join("\n");

    expect(markup).toContain("Loading this view");
    expect(markup).toContain("Live data");
    expect(markup).toContain("Data is stale. Recommendations are paused until fresh prices arrive.");
    expect(markup).toContain("Data is degraded");
    expect(markup).toContain("Nothing to show yet");
    expect(markup).toContain("This view hit an error");
  });
});
