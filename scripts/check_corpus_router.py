from __future__ import annotations

import json
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
ROUTER_FILES = [
    REPO_ROOT / "data" / "relations" / "relation_router.v1.json",
    REPO_ROOT / "data" / "corpus" / "corpus_router.v1.json",
]
SKILL_DIR = REPO_ROOT / "docs" / "agent-skills" / "grandedge-corpus-router"
SKILL_FILE = SKILL_DIR / "SKILL.md"
REFERENCE_FILES = [
    SKILL_DIR / "references" / "router.md",
    SKILL_DIR / "references" / "corpus-map.md",
]
DOC_FILES = [
    REPO_ROOT / "docs" / "corpus" / "README.md",
    REPO_ROOT / "docs" / "corpus" / "router.md",
    REPO_ROOT / "docs" / "corpus" / "relation-corpus.md",
    REPO_ROOT / "docs" / "corpus" / "market-intelligence-corpus.md",
    REPO_ROOT / "docs" / "corpus" / "review-workflow.md",
    REPO_ROOT / "docs" / "corpus" / "token-budgeting.md",
    REPO_ROOT / "docs" / "corpus" / "examples.md",
    REPO_ROOT / "docs" / "corpus" / "source-policy.md",
]
PLANNED_SOURCE_FILES = {
    "data/relations/source_registry.v1.json",
    "data/relations/item_sets.v1.json",
    "data/relations/recipes.v1.json",
    "data/relations/repairs.v1.json",
    "data/relations/alchemy.v1.json",
    "data/relations/dose_decant.v1.json",
    "data/relations/charge_links.v1.json",
    "data/relations/degrade_links.v1.json",
    "data/relations/categories.v1.json",
    "data/relations/substitutes.v1.json",
    "data/relations/market_analysis_sources.v1.json",
    "data/corpus/source_registry.v1.json",
    "data/corpus/market_analysis.v1.json",
    "data/corpus/events.v1.json",
    "data/corpus/competitor_capabilities.v1.json",
    "data/corpus/review_notes.v1.json",
}


def ensure(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def validate_router_file(path: Path) -> None:
    routes = json.loads(path.read_text(encoding="utf-8"))
    ensure(isinstance(routes, list) and routes, f"{path} must contain a non-empty JSON array")

    for route in routes:
        route_id = route.get("route_id", "<missing>")
        ensure(route.get("max_files_to_open", 0) > 0, f"{route_id} must allow at least one file")
        ensure(route.get("max_sample_entries", 0) > 0, f"{route_id} must allow at least one sample entry")
        ensure(bool(route.get("stop_and_ask")), f"{route_id} must declare stop conditions")
        ensure(route.get("forbidden_full_load") is True, f"{route_id} must forbid full-load usage")

        for doc in route.get("read_first", []):
            ensure((REPO_ROOT / doc).exists(), f"{route_id} references missing doc {doc}")

        for source_glob in route.get("source_globs", []):
            if "*" not in source_glob:
                ensure(
                    source_glob in PLANNED_SOURCE_FILES or (REPO_ROOT / source_glob).exists(),
                    f"{route_id} references unknown source glob {source_glob}",
                )


def validate_skill_frontmatter() -> None:
    text = SKILL_FILE.read_text(encoding="utf-8")
    match = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
    ensure(match is not None, "SKILL.md must start with YAML frontmatter")
    lines = [line for line in match.group(1).splitlines() if line.strip()]
    keys = [line.split(":", 1)[0].strip() for line in lines]
    ensure(keys == ["name", "description"], "SKILL.md frontmatter must contain only name and description")
    name = lines[0].split(":", 1)[1].strip()
    ensure(re.fullmatch(r"[a-z0-9-]+", name), "Skill name must be lowercase hyphen-case")


def validate_references() -> None:
    for path in REFERENCE_FILES:
        text = path.read_text(encoding="utf-8")
        ensure('"entries": [' not in text, f"{path} must not embed a full JSON corpus array")
        ensure("[" not in text or "route" in text.lower() or "start here" in text.lower(), f"{path} should stay pointer-like")


def main() -> int:
    for path in DOC_FILES + [SKILL_FILE] + REFERENCE_FILES + ROUTER_FILES:
        ensure(path.exists(), f"Missing required file: {path}")

    for path in ROUTER_FILES:
        validate_router_file(path)

    validate_skill_frontmatter()
    validate_references()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # noqa: BLE001
        print(f"corpus router check failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
