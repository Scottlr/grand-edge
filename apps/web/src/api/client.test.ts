import { describe, expect, it, vi } from "vitest";

import { ApiClient } from "./client";

describe("ApiClient", () => {
  it("preserves opaque icon CDN URLs from API fixtures", async () => {
    const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(
        JSON.stringify([
          {
            itemId: 1,
            name: "Chef's hat",
            examine: null,
            members: false,
            buyLimit: null,
            lowAlch: null,
            highAlch: null,
            value: null,
            icon: {
              sourceFileName: "Chef's_hat.png",
              canonicalFileName: "Chef's_hat.png",
              cdnUrl: "https://oldschool.runescape.wiki/images/Chef%27s_hat.png",
              source: "mapping_icon",
            },
          },
          {
            itemId: 2,
            name: "Mining cape(t)",
            examine: null,
            members: true,
            buyLimit: null,
            lowAlch: null,
            highAlch: null,
            value: null,
            icon: {
              sourceFileName: "Mining_cape(t).png",
              canonicalFileName: "Mining_cape(t).png",
              cdnUrl: "https://oldschool.runescape.wiki/images/Mining_cape%28t%29.png",
              source: "mapping_icon",
            },
          },
        ]),
        {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
        },
      ),
    );

    const client = new ApiClient({ baseUrl: "https://api.example.test", fetchImpl });
    const items = await client.getItems({ limit: 24, offset: 0 });

    expect(items[0]?.icon?.cdnUrl).toBe("https://oldschool.runescape.wiki/images/Chef%27s_hat.png");
    expect(items[1]?.icon?.cdnUrl).toBe("https://oldschool.runescape.wiki/images/Mining_cape%28t%29.png");
  });

  it("patches strategy state through the backend route", async () => {
    const fetchImpl = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(JSON.stringify({ strategyId: "kalman", enabled: false }), {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      }),
    );

    const client = new ApiClient({ baseUrl: "https://api.example.test", fetchImpl });
    await client.patchStrategy("kalman", { enabled: false });

    expect(fetchImpl).toHaveBeenCalledTimes(1);
    const [url, init] = fetchImpl.mock.calls[0] ?? [];
    expect(url).toBe("https://api.example.test/api/strategies/kalman");
    expect(init?.method).toBe("PATCH");
    expect(init?.body).toBe(JSON.stringify({ enabled: false }));
    expect((init?.headers as Record<string, string>)["Content-Type"]).toBe("application/json");
  });
});
