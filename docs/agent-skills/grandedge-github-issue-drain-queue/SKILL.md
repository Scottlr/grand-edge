---
name: grandedge-github-issue-drain-queue
description: Drain eligible GitHub issues for Scottlr/grand-edge by repeatedly selecting one unblocked issue, completing it fully, and stopping only at EMPTY_QUEUE or BLOCKED.
---

# GrandEdge GitHub Issue Drain Queue

Drain the `Scottlr/grand-edge` issue queue one issue at a time.

## Read First

- `AGENTS.md`
- `docs/agent-skills/grandedge-github-issue-selector/SKILL.md`
- `docs/agent-skills/grandedge-github-issue-complete-one/SKILL.md`
- `docs/agent-skills/grandedge-github-issue-loop/SKILL.md`

## Workflow

1. Use `grandedge-github-issue-selector` to select exactly one eligible issue.
2. If the selector returns `EMPTY_QUEUE`, stop and report queue drained.
3. If the selector returns `BLOCKED`, stop and report the blocker.
4. Use `grandedge-github-issue-complete-one` to complete the selected issue.
5. After cleanup, return to step 1.

## Repository Rules

- Only operate on `Scottlr/grand-edge`.
- Every `gh issue`, `gh pr`, and `gh repo` command must target
  `Scottlr/grand-edge` explicitly or be run from a verified checkout whose
  origin is `https://github.com/Scottlr/grand-edge.git`.
- Do not run multiple issue implementations in the same checkout at once.
- Stop if the worktree is dirty with unrelated changes after completing an
  issue.

## Stop Conditions

- `EMPTY_QUEUE`: no unblocked open issue remains.
- `BLOCKED`: remaining issues have unresolved blockers.
- Permission failure: cannot push, create PR, merge, close issue, or delete
  branch.
- Verification failure that cannot be fixed within the selected issue scope.
- Dirty worktree contains unrelated user changes that make the next issue unsafe.
