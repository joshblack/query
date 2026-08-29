# Use isolated lanes for parallel project work

Date: 2026-08-29

The project workflow needs to support parallel research, implementation, and
review without allowing concurrent agents to overwrite one another or weakening
the atomic GitHub stack used to land a project.

## Decision

Use at most five active sub-agents across all project roles. Planning and
research fan out into mutually exclusive lanes and return to a designated
architect or product manager for synthesis.

For implementation, create one staging branch, worktree, and artifact directory
per independent task. Each implementer has an exclusive write lease and returns
task-scoped local commits. The orchestrator imports completed commits in
dependency order through one canonical integration worktree and GitHub stack.
Only canonical stack branches receive pull requests.

Keep GitHub issues, Project fields, checkpoints, pull requests, and tracked files
as the durable source of truth. Store persistent local supporting artifacts
under the ignored `.tmp/projects/<project-key>/` directory in the primary
checkout.

### Impact

Independent implementation tasks can run concurrently without sharing a working
tree or mutating the live stack. Immutable staging heads make interrupted work
recoverable and give the orchestrator deterministic integration inputs.

Serial canonical integration adds a cherry-pick and validation step, but
preserves one task per pull request, dependency-ordered review, and atomic stack
landing. The five-slot limit applies across planning, research, implementation,
synthesis, and review, so dispatch may intentionally use fewer writers when
integration is the bottleneck.

### Alternatives

- **Edit separate layers of the live stack concurrently.** This avoids
  cherry-picking, but a lower-layer rewrite mutates every branch above it and
  invalidates concurrent work and reviews.
- **Create one independent pull request per staging branch.** This simplifies
  fan-out, but loses the canonical dependency order and atomic project landing.
- **Create multiple stacks by default.** This supports independent merges, but
  makes partial delivery, rollback, and checkpoint recovery more complex.
  Multiple stacks remain an explicit exception when partial landing is
  acceptable.
- **Leave staging work uncommitted.** This minimizes implementer Git authority,
  but makes concurrent recovery and deterministic integration fragile.

## Links and resources

- [Project skill](../../.github/skills/project/SKILL.md)
- [Project workflow](../../.github/skills/project/references/workflow.md)
