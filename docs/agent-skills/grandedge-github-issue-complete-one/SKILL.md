---
name: grandedge-github-issue-complete-one
description: Complete exactly one selected GitHub issue from Scottlr/grand-edge through branch, implementation, verification, pull request, merge, issue close, and cleanup.
---

# GrandEdge GitHub Issue Complete One

Complete exactly one selected issue in `Scottlr/grand-edge`. Use this only
after an issue number has been selected or explicitly provided by the user.

## Read First

- `AGENTS.md`
- `docs/agent-skills/grandedge-github-issue-loop/SKILL.md`
- The selected issue body and comments:

```powershell
gh issue view <number> --repo Scottlr/grand-edge --json number,title,url,body,labels,comments
```

## Required Flow

1. Verify the issue is open, unblocked, and not already claimed.
2. Add `in-progress` to claim it:

```powershell
gh issue edit <number> --repo Scottlr/grand-edge --add-label in-progress
```

3. Create exactly one branch from the default branch:

```powershell
$DEFAULT_BRANCH = gh repo view Scottlr/grand-edge --json defaultBranchRef --jq .defaultBranchRef.name
git fetch origin
git switch $DEFAULT_BRANCH
git pull --ff-only
git switch -c codex/issue-<number>-<short-slug>
```

4. Implement the smallest correct change for the issue.
5. Run focused verification, then broader checks if risk warrants it.
6. Stage only explicit files related to the issue.
7. Commit, push, and open a PR that references the issue.
8. Wait for checks where available.
9. Merge only when verification is sufficient and policy allows it.
10. Close the issue with a comment linking the merged PR.
11. Delete remote/local issue branches and return to the default branch.

## Guardrails

- Never use `git add .` or `git add -A`.
- Never commit to `main`, `master`, or the default branch.
- Never use `git reset --hard`, `git clean -fd`, force-push, or history rewrite.
- Do not stage generated caches, logs, secrets, `target/`, or unrelated work.
- Do not close the issue before the PR is merged.
- Stop and report if permissions prevent push, PR creation, merge, close, or cleanup.

## Completion Report

End with:

- issue number and title;
- branch and PR URL;
- files changed;
- verification commands and results;
- merge/close/cleanup status;
- blockers if any step could not finish.
