import { describe, expect, it } from "vitest";

import { forbiddenTrustCopy } from "./alerts";
import { findForbiddenDefaultUiTerms } from "./copyRules";

const sourceModules = import.meta.glob(
  [
    "./*.ts",
    "../styles.css",
    "../layout/*.tsx",
    "../views/*.tsx",
    "../features/**/*.tsx",
    "../components/**/*.tsx",
  ],
  {
    eager: true,
    import: "default",
    query: "?raw",
  },
) as Record<string, string>;

const allowedTrustFiles = new Set([
  "./alerts.ts",
  "./glossary.ts",
]);

const allowedDefaultTermFiles = new Set([
  "./alerts.ts",
  "./copyRules.ts",
  "./glossary.ts",
  "./plainLanguage.ts",
]);

function userFacingFiles() {
  return Object.entries(sourceModules)
    .filter(([path]) => !path.includes(".test."))
    .filter(([path]) => !path.endsWith("styles.css"))
    .filter(([path]) => !path.includes("/domain/"))
    .filter(([path]) => !path.includes("/api/"))
    .filter(([path]) => !path.includes("/charts/"))
    .filter(([path]) => !path.includes("/navigation/"));
}

function userVisibleStringsFromSource(source: string) {
  const matches = source.matchAll(/"([^"\n]{3,})"|'([^'\n]{3,})'|`([^`\n]{3,})`/g);

  return Array.from(matches, (match) => {
    const value = match[1] ?? match[2] ?? match[3] ?? "";
    return value.trim();
  }).filter((value) => /[A-Za-z]/.test(value) && value.includes(" "));
}

function allowsAdvancedTerms(path: string) {
  return (
    path.includes("/strategy-lab/") ||
    path.includes("/model-accuracy/") ||
    path.includes("/account/") ||
    path.includes("LearnModal.tsx") ||
    path.includes("GlossaryProvider.tsx")
  );
}

describe("copy safety", () => {
  it("rejects forbidden trust terms outside rule and learn sources", () => {
    const violations = userFacingFiles().flatMap(([path, source]) => {
      if (allowedTrustFiles.has(path)) {
        return [];
      }

      return userVisibleStringsFromSource(source).flatMap((value) =>
        forbiddenTrustCopy
          .filter((term) => value.toLowerCase().includes(term))
          .map((term) => `${path}: ${term}`),
      );
    });

    expect(violations).toEqual([]);
  });

  it("rejects forbidden default UI jargon outside learn and advanced sources", () => {
    const violations = userFacingFiles().flatMap(([path, source]) => {
      if (allowedDefaultTermFiles.has(path) || allowsAdvancedTerms(path)) {
        return [];
      }

      return userVisibleStringsFromSource(source).flatMap((value) =>
        findForbiddenDefaultUiTerms(value).map((violation) => `${path}: ${violation.term}`),
      );
    });

    expect(violations).toEqual([]);
  });
});
