---
name: grandedge-github-issue-writing
description: Write GrandEdge feature-planned tasks from features/*/tasks.md and linked features/*/tasks/T###.md task files to GitHub issues for Scottlr/grand-edge, including blocked-by/blocking dependency relationships, meaningful labels, and task table issue links.
---

# Skill: grandedge-github-issue-writing

# GitHub Issue Writing

Write feature-planned tasks from a compact `features/*/tasks.md` index and its
linked `features/*/tasks/T###.md` task detail files to GitHub issues in the
**Scottlr/grand-edge** repository. Set dependency relationships (blocked by and
blocking), meaningful labels, and task table issue links.

## Guardrails

- **Only ever write to `Scottlr/grand-edge`.** Never create, edit, or
  modify issues in any other repository. If the user asks to write issues
  elsewhere, refuse and state this constraint.
- Use the `gh` CLI exclusively.
- Read the planning index first (`features/{feature-name}/tasks.md`) to get
  task IDs, titles, descriptions, dependency info, and `Task File` links.
- Read each linked `tasks/T###.md` file for the full issue body. Do not infer
  issue bodies from table summaries.
- Create issues in dependency order: independent tasks first, then dependent
  ones.

## Critical Rules

1. **`gh issue edit --body` REPLACES the entire body.** It does not append.
   Always construct the full body content before calling `gh issue edit`.
2. **Never use `2>&1` with `gh api graphql`.** It corrupts JSON output by mixing
   stderr text into stdout. Parse the raw output directly.
3. **`addIssueDependency` does not exist.** Use `addBlockedBy` for GitHub-native
   sidebar dependencies.
4. **Write bodies to temp files first.** Use `--body-file` instead of inline
   `--body` to avoid PowerShell/shell escaping issues with large bodies.
5. **Fetch node IDs in a loop, store them, then apply mutations.** Do not
   interleave fetching and mutating in the same pipeline.

## Workflow

### 1. Get Repository IDs

Before creating or editing issues, get the repository-level IDs needed for
labels and issue types:

```sh
gh api graphql \
  -f owner="Scottlr" \
  -f repo="grand-edge" \
  -f query='
query($owner: String!, $repo: String!) {
  repository(owner: $owner, name: $repo) {
    id
    labels(first: 100) { nodes { id name } }
    issueTypes(first: 20) { nodes { id name } }
  }
}'
```

Save the returned IDs for reuse. Match labels and issue types by `name`. The
GrandEdge repository may not have issue types enabled; if `issueTypes` is null
or empty, skip issue type assignment and continue.

Ensure these labels exist before creating issues. Create missing labels with
`gh label create --repo Scottlr/grand-edge` using the listed colors and
descriptions:

| Label | Color | Use |
|---|---:|---|
| `enhancement` | `a2eeef` | Default feature/task issue label. |
| `documentation` | `0075ca` | Docs, runbooks, corpus docs, skill docs. |
| `backend` | `5319e7` | Rust API, storage, ingestion, recommender, simulator, metrics. |
| `frontend` | `1d76db` | React/Vite UI, charts, accessibility, copy QA. |
| `rust` | `dea584` | Rust crate/API/runtime implementation. |
| `python` | `3572A5` | Python ML research/export work. |
| `ml` | `6f42c1` | Model artifacts, runtime, graph-aware ML, evaluation. |
| `graph` | `0e8a16` | Item graph, relations, learned edges, blast radius. |
| `corpus` | `fbca04` | Relation/event corpus, source policy, router skills. |
| `evidence` | `006b75` | Evidence ledger, reason atoms, outcomes, reconstruction. |
| `testing` | `d4c5f9` | Test harnesses, benchmarks, QA gates. |
| `devex` | `c2e0c6` | Scripts, runbooks, local/Docker workflows, tooling. |
| `api` | `0052cc` | Axum/OpenAPI/routes/DTO/view contracts. |
| `blocked` | `b60205` | Optional marker for issues with unresolved blockers. |
| `in-progress` | `fbca04` | Queue-drain/implementation claim marker. |
| `pr-taken` | `fbca04` | Prevents two agents taking the same issue. |

Apply labels from the task contents:

- Always add `enhancement` unless the task is docs-only, then use
  `documentation`.
- Add `backend`/`rust` for `crates/`, `Cargo.toml`, storage, ingestion,
  recommender, simulator, metrics, API, model runtime, or analytics work.
- Add `frontend` for `apps/web`, routes, components, charts, accessibility, or
  copy-safety work.
- Add `api` for `crates/api`, OpenAPI, route/view contracts, live streams, or
  schema surfaces.
- Add `python` and `ml` for `/ml`, artifacts, training/export/evaluation, or
  PyO3 research bindings.
- Add `graph` for item graph, relations, learned edges, graph features, linked
  items, blast-radius, or graph ML.
- Add `corpus` for `data/relations`, `data/corpus`, `docs/corpus`, or
  repo-local corpus skills.
- Add `evidence` for evidence ledger, reason atoms, prediction links, outcomes,
  reason metrics, retention, or reconstruction.
- Add `testing` for property/snapshot/container/benchmark/browser QA tasks.
- Add `devex` for CLI/config/observability, local scripts, Docker, runbooks, or
  repo workflow skills.
- Add `blocked` when the task has one or more unresolved blockers.

### 2. Parse the Planning File

Read `features/{feature-name}/tasks.md`. Extract from the dependency table:
- Task ID (T001, T002, etc.)
- Completed marker; skip tasks already marked `[x]` if creating issues only for
  remaining implementation work
- Title
- Description
- Blocked By column (other task IDs)
- Task File link (`tasks/T###.md`)

Validate that each linked task file exists and starts with a heading matching
the table ID and title. If a legacy plan has no `Task File` column, stop and
ask whether to migrate the plan before creating issues; do not create issues
from table summaries alone.

### 3. Create Issues in Dependency Order

Create independent tasks first (Blocked By = None), then tasks whose blockers
already exist.

For each task, create the issue with a body constructed from the linked
`tasks/T###.md` file. The body **must** contain both:

- `## Blocked By` with a list of issue numbers that block this task, or `None`;
- `## Blocking` with a list of issue numbers this task blocks, or `None`.

GitHub auto-parses `## Blocked By` issue links into the sidebar dependency
feature. The `## Blocking` section is for humans and queue-selection agents.
If the task file has a `#### Blocked By` section with task IDs, replace or
supplement it in the issue body with the GitHub issue numbers that now exist.

```sh
gh issue create --repo Scottlr/grand-edge --title "Task Title" --body-file /path/to/issue-body.md
```

After creation, record the issue number and node ID for dependency linking.

### 4. Set Labels

Add labels to each issue using `gh issue edit`:

```sh
gh issue edit <number> --repo Scottlr/grand-edge --add-label "enhancement"
```

Use the label names that match the task category. Common labels:
- `enhancement` — new feature work
- `bug` — defect fixes
- `documentation` — docs work
- `refactor` — internal improvements

### 5. Set Issue Type

Set the issue type using GraphQL:

```sh
gh api graphql \
  -f issueId="<ISSUE_NODE_ID>" \
  -f issueTypeId="<ISSUE_TYPE_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $issueTypeId: ID!) {
  updateIssueIssueType(input: {
    issueId: $issueId,
    issueTypeId: $issueTypeId
  }) {
    issue { number title issueType { name } }
  }
}'
```

### 6. Set Dependency Relationships

Use `addBlockedBy` to set the GitHub-native dependency sidebar. For each task
with blockers:

```sh
gh api graphql \
  -f issueId="<BLOCKED_ISSUE_NODE_ID>" \
  -f blockingIssueId="<BLOCKING_ISSUE_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $blockingIssueId: ID!) {
  addBlockedBy(input: {
    issueId: $issueId,
    blockingIssueId: $blockingIssueId
  }) {
    issue { number title }
  }
}'
```

**Meaning:** `issueId` is the issue being blocked. `blockingIssueId` is the
issue that blocks it.

The inverse mutation `addBlocking` is also available:

```sh
gh api graphql \
  -f issueId="<BLOCKER_ISSUE_NODE_ID>" \
  -f blockedIssueId="<BLOCKED_ISSUE_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $blockedIssueId: ID!) {
  addBlocking(input: {
    issueId: $issueId,
    blockedIssueId: $blockedIssueId
  }) {
    issue { number title }
  }
}'
```

Both produce the same sidebar relationship. Use `addBlockedBy` consistently.

### 7. Remove a Dependency

```sh
gh api graphql \
  -f issueId="<BLOCKED_ISSUE_NODE_ID>" \
  -f blockingIssueId="<BLOCKING_ISSUE_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $blockingIssueId: ID!) {
  removeBlockedBy(input: {
    issueId: $issueId,
    blockingIssueId: $blockingIssueId
  }) {
    issue { number title }
  }
}'
```

The inverse removal `removeBlocking` is also available:

```sh
gh api graphql \
  -f issueId="<BLOCKER_ISSUE_NODE_ID>" \
  -f blockedIssueId="<BLOCKED_ISSUE_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $blockedIssueId: ID!) {
  removeBlocking(input: {
    issueId: $issueId,
    blockedIssueId: $blockedIssueId
  }) {
    issue { number title }
  }
}'
```

### 7b. Batch-Fetch Issue Node IDs

When wiring many dependencies, fetch all node IDs first, store them, then apply
mutations:

```sh
# Step 1: Fetch all node IDs (one per issue)
gh api graphql \
  -F owner="Scottlr" \
  -F repo="grand-edge" \
  -F num=123 \
  -f query='
query($owner: String!, $repo: String!, $num: Int!) {
  repository(owner: $owner, name: $repo) {
    issue(number: $num) { id number title }
  }
}'
```

Loop over each issue number, collect the `id` field into a map, then use those
IDs in `addBlockedBy` mutations. **Never use `2>&1`** — it corrupts JSON.

### 7c. Labels via GraphQL

For bulk label operations, use `addLabelsToLabelable`:

```sh
gh api graphql \
  -f issueId="<ISSUE_NODE_ID>" \
  -f labelIds[]="<LABEL_NODE_ID>" \
  -f query='
mutation($issueId: ID!, $labelIds: [ID!]!) {
  addLabelsToLabelable(input: {
    labelableId: $issueId,
    labelIds: $labelIds
  }) {
    labelable {
      ... on Issue { number title }
    }
  }
}'
```

For simple cases, `gh issue edit` is easier:

```sh
gh issue edit <number> --repo Scottlr/grand-edge --add-label "enhancement,priority:high"
gh issue edit <number> --repo Scottlr/grand-edge --remove-label "enhancement"
```

### 8. Update the Planning File

After all issues are created, update `features/{feature-name}/tasks.md` to fill
in the `Github Issue #` column with markdown links to the created issues.

## Issue Body Format

Each issue body starts from the linked `tasks/T###.md` task detail file. Normalize
heading depth if needed, but preserve the task's objective, scope, touchpoints,
implementation details, invariants, acceptance criteria, testing guidance, and
handoff context.

GitHub auto-parses `## Blocked By` headings into the sidebar dependency feature.
The final issue body **must** include both `## Blocked By` and `## Blocking`:

```markdown
## Objective

One sentence describing what this task accomplishes.

## Scope

Bullet list of concrete deliverables.

## Out of Scope

Bullet list of what this task must not do.

## Blocked By

- #NNN
- #NNN

## Blocking

- #NNN
- #NNN

## Implementation Touchpoints

Detailed guidance for the implementing agent.

## Invariants

Rules that must remain true after this task.

## Acceptance Criteria

- [ ] Verifiable checkbox items.

## Testing Guidance

What tests should be added or changed.

## Agent Handoff Context

Files to touch, patterns to follow, extension points.
```

The `## Blocked By` heading with `- #NNN` list items is what GitHub parses into
the sidebar. `## Blocking` documents the inverse relationship for humans and
queue agents. Use `None` under either heading when no linked issues exist. Do
not use `**Blocked by:**` bold text or other formats.

## Quick Reference

| Action | Command |
|--------|---------|
| Create issue | `gh issue create --repo Scottlr/grand-edge --title "..." --body-file path.md` |
| Add labels | `gh issue edit <n> --repo Scottlr/grand-edge --add-label "label1,label2"` |
| Set issue type | `gh api graphql ... updateIssueIssueType` |
| Add blocked-by | `gh api graphql ... addBlockedBy` |
| Add blocking | `gh api graphql ... addBlocking` |
| Remove blocked-by | `gh api graphql ... removeBlockedBy` |
| Remove blocking | `gh api graphql ... removeBlocking` |
| Get repo IDs | `gh api graphql ... repository { labels issueTypes }` |
| Get issue node ID | `gh issue view <n> --repo Scottlr/grand-edge --json id --jq .id` |

## Repository Constraint

All issue operations target **Scottlr/grand-edge** only. The `gh`
CLI must be run with the correct repo context. If the current directory is not
the `grand-edge` checkout, use `--repo Scottlr/grand-edge` on all
`gh issue` commands.

