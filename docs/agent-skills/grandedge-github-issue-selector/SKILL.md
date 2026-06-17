---
name: grandedge-github-issue-selector
description: Select exactly one eligible unblocked GitHub issue from Scottlr/grand-edge for implementation, or return EMPTY_QUEUE, without modifying files or GitHub state.
---

# GrandEdge GitHub Issue Selector

Select one open issue from `Scottlr/grand-edge` that is ready for implementation.
This skill is read-only: do not edit files, labels, branches, issues, or pull
requests.

## Workflow

1. Verify repository access:

```powershell
gh repo view Scottlr/grand-edge --json nameWithOwner,defaultBranchRef,url
gh auth status
```

2. List eligible issues:

```powershell
gh issue list `
  --repo Scottlr/grand-edge `
  --state open `
  --search "is:issue is:open -is:blocked -label:in-progress -label:pr-taken" `
  --json number,title,url,labels,assignees,milestone,updatedAt `
  --limit 20
```

3. Prefer issues in dependency order:

- Issues with no GitHub-native blockers.
- Issues without `in-progress` or `pr-taken`.
- Lower `T###` task IDs before higher IDs when both are ready.
- Smaller/foundational issues before broad polish issues.

4. Return exactly one result:

```text
SELECTED #<number> - <title>
Reason: <why this is eligible now>
```

If no eligible issue exists, return:

```text
EMPTY_QUEUE
Reason: no open unblocked issue without in-progress/pr-taken labels.
```

If all remaining issues are blocked but GitHub search does not expose why,
return:

```text
BLOCKED
Reason: <specific observed blocker>
```

## Guardrails

- Do not claim or label the issue.
- Do not open a branch.
- Do not edit the issue body.
- Do not select an issue with unresolved blockers.
- Do not select an issue already marked `in-progress` or `pr-taken`.
- Use only `Scottlr/grand-edge`.
