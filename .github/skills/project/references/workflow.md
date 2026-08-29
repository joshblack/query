# Project workflow

GitHub issues are the durable project ledger. Conversation history, local notes,
and agent state are supporting context, not the source of truth.

Issue labels classify the kind of work. The
[configured GitHub Project](./config.md) owns lifecycle state through its
built-in `Status` field. Do not encode lifecycle state in labels.

Each project lands as one canonical GitHub stack. Parallel implementation occurs
on isolated staging branches in dedicated worktrees, then the orchestrator
serializes completed commits through one canonical integration worktree. The
primary repository checkout remains available for user work and stores ignored
temporary project artifacts under `.agents/tmp/projects/`.

Each integrated task becomes one branch and pull request in the canonical stack,
ordered from foundational work at the bottom to dependent work at the top. Land
the project with the stack merge command rather than merging its pull requests
or staging branches individually.

## Ownership

The orchestrator is the only participant that creates or updates issues,
checkpoints, links, type labels, and GitHub Project fields. Sub-agents inspect,
implement, review, and return structured handoffs.

This ownership rule prevents concurrent agents from overwriting each other and
makes the project understandable from GitHub alone.

## Concurrency and leases

Use one orchestrator-owned limit of five active sub-agents across the project.
Product managers, architects, researchers, implementers, reviewers, synthesis
agents, and integration-remediation agents each consume one slot from dispatch
until they return a complete, blocked, failed, or acknowledged cancellation
handoff.

The orchestrator, queued assignments, completed handoffs, and direct GitHub
operations do not consume slots. Do not reserve a slot for synthesis or
integration. Dispatch fewer than five agents when integration or review capacity
is the bottleneck.

Sub-agents must not delegate further. Every editing agent receives one exclusive
worktree and branch lease. No two editing agents may use the same worktree,
branch, or canonical stack at the same time. Read-only reviews may run in
parallel against immutable base and head SHAs.

## Instructions and skills

Every delegation must provide the repository root, worktree if applicable,
expected branch and base SHA, scope, expected paths, relevant skills, validation
expectations, write lease, project artifact root, and assigned artifact
directory.

Before substantive work, each agent must:

1. Read the root `AGENTS.md`.
2. Discover and read each nested `AGENTS.md` that applies to its scope.
3. Read its agent contract.
4. Read the `SKILL.md` and required references for every skill it uses.
5. Summarize the setup, style, validation, ownership, and scope obligations.
6. Re-check for nested instructions before entering a new subtree.
7. Stop as blocked when required instructions are unavailable, contradictory,
   or impossible to follow.

All agents may use available repository-local and global skills. Skills provide
instructions and reusable workflows, but do not expand tools, scope, write
leases, branch authority, concurrency slots, or ownership of GitHub state.

Every handoff must identify the instruction and skill files loaded, derived
obligations, newly checked subtrees, environment setup, exact validation
requirements, and any conflicts. For implementation, compare the inventory with
the actual changed paths and reject the handoff if instructions or required
validation were missed.

For this repository, implementation validation is derived from `AGENTS.md` and
defaults to:

```sh
mise run fmt
mise run lint
mise run test
```

Use focused variants only when the task supports them without changing workspace
feature resolution. Otherwise, run the full task. Validation at a staging head
does not replace validation after canonical integration.

## Planning and research fan-out

Use a frame, fan-out, and synthesis protocol:

1. Record the repository commit, shared question, constraints, evidence
   requirements, evaluation rubric, output format, and synthesis owner.
2. Assign mutually exclusive lanes with a unique question or hypothesis,
   explicit exclusions, and awareness of the other lane assignments.
3. Use the product manager for outcomes and scope, the architect for system
   constraints and consequential options, and researchers for bounded evidence
   gathering.
4. Require citations, disconfirming evidence, unresolved unknowns, and
   confidence from each lane.
5. After the fan-out completes, use an architect on `gpt-5.6-sol` for technical
   synthesis or the product manager on `gpt-5.6-sol` when product judgment
   dominates.
6. Reconcile conflicts against the shared rubric and produce one
   dependency-aware recommendation before changing durable project state.

Use `gpt-5.6-luna` for bounded research, implementation, and review. Escalate a
research lane to the architect only when it requires cross-system judgment.

## Temporary project artifacts

Before fan-out, invoke the `tmp` skill and create an artifact root in the primary
checkout:

```text
<primary-repository-root>/.agents/tmp/projects/<project-key>/
```

Use the parent issue number when available and a stable descriptive key before
an issue exists. Pass the absolute path to every agent. Use these ownership
boundaries:

```text
.agents/tmp/projects/<project-key>/
├── orchestrator/
├── agents/<lane-id>/
├── pr-bodies/
└── generated/
```

The orchestrator owns shared synthesis, integration logs, pull request bodies,
and cleanup. Each sub-agent may write only within its assigned lane directory
and must report when each artifact is safe to remove. Source inspection, edits,
Git operations, and validation remain in the assigned code worktree.

Do not use OS `/tmp`, session temporary directories, task worktrees, `.git/`, or
`target/` for plans, research, handoffs, validation logs, generated pull request
bodies, or other uncommitted artifacts used by the workflow.
Disposable one-command intermediates may use ephemeral storage only when no
handoff depends on them.

GitHub issues, Project fields, checkpoints, pull requests, and tracked files
remain the durable source of truth. Promote essential conclusions, decisions,
blockers, and next actions to GitHub before ending a session. Never store
credentials, tokens, environment secrets, or unnecessary sensitive output in
the artifact root.

Every handoff must report its artifact root, assigned directory, created or
updated files, disposable files, state to promote to GitHub, and cleanup
readiness. Reject handoffs that depend on missing or ephemeral files or write
temporary artifacts outside their assigned directory.

Clean up artifacts incrementally:

1. Remove a lane's reports and logs after its handoff has been consumed,
   required evidence has been reviewed, and its essential state is in GitHub.
2. Remove generated pull request body files after confirming GitHub has the
   current body and no pending update depends on the local file.
3. Remove abandoned lane artifacts after preserving recoverable work and
   recording the cancellation outcome.
4. Before every checkpoint and session end, inspect the exact project artifact
   directory and remove files and empty directories that no active lane,
   pending integration, review, or recovery action needs.
5. Record every retained artifact and why it is still required.

Remove only specific, resolved files or directories. Never delete the
`.agents/tmp/` root, use cleanup globs, or retain artifacts merely because they
might be useful later.

## Parent project issue

Apply the `type: project` label and include this marker:

```markdown
<!-- copilot-project -->
```

Use this body shape:

```markdown
<!-- copilot-project -->

The current state and the outcome this project tracks.

## Outcome

The user-visible result and why it matters.

## Success criteria

- [ ] An observable result.

## Scope

### Requirements

- A requirement.

### Non-goals

- A boundary.

### Constraints

- A constraint.

## Decisions

- A decision or a link to a decision record.

## Work

- [ ] #123

## Candidate work

- Work that is not understood or committed enough for a task issue.

<!-- project-state:start -->
## Current state

- **Phase:** Planning, implementing, integrating, in review, blocked, or done
- **Integration worktree:** An absolute path, or not created
- **Artifact root:** An absolute path, or not created
- **Canonical stack:** The stack number, base SHA, and branch chain, or not submitted
- **Active lanes:** Task, worktree, branch, base and head SHAs, and phase, or none
- **Parked lanes:** Completed, blocked, or cancelled staging lanes, or none
- **Pull requests:** Pull requests under review with base and head SHAs, or none
- **Last checkpoint:** YYYY-MM-DD
- **Next action:** The next planning, lane, integration, or review action.
- **Blockers:** None, or linked blockers and the decision needed.
<!-- project-state:end -->
```

The orchestrator may update the managed current state block in place. Preserve
all content outside its markers.

## Task issues

Apply exactly one of these type labels:

- `type: feature` for a user-visible capability or meaningful enhancement.
- `type: bug` for behavior that does not work as intended.
- `type: task` for enabling, maintenance, investigation, or implementation work
  that is not independently a feature or bug.

Use this marker, replacing the parent number:

```markdown
<!-- copilot-project-task parent=#123 -->
```

Each task must be independently reviewable and should produce useful progress.
Use this body shape:

```markdown
<!-- copilot-project-task parent=#123 -->

This issue tracks the result this slice delivers.

## Context

Why this slice exists and what project decision or constraint applies.

## Acceptance criteria

- [ ] An observable criterion.

## Validation

- The checks that demonstrate the criteria are met.

## Out of scope

- Related work that does not belong in this slice.
```

Link task issues from the parent issue with a task list. Use native sub-issues
when available, but do not make the workflow depend on them.

Record task ordering with native `blocked by` and `blocking` issue
relationships. Do not duplicate dependency lists in issue bodies. A task is
unblocked only when it has no open `blocked by` relationships.

## GitHub Project

Add every parent and task issue to the repository's configured GitHub Project.
Use the built-in `Status` field as the lifecycle source of truth:

- `Backlog`: Defined work that is not actively being implemented.
- `In Progress`: A task with an active staging or integration lane.
- `In Review`: Implementation is complete and review is active.
- `Blocked`: The issue has one or more open native `blocked by` relationships,
  or the checkpoint identifies a decision that prevents progress.
- `Done`: Acceptance criteria and validation are complete.

New items start in `Backlog`. Select a task only when its acceptance criteria
are actionable and it has no open `blocked by` relationships. When a blocker is
added, move the issue to `Blocked`. When all blockers close, move it back to
`Backlog`.

Multiple child tasks may be `In Progress` or `In Review` when they have exclusive
lanes or canonical pull requests. The parent project issue remains `In Progress`
while any viable lane, integration, or review work is active. Set it to `Blocked`
only when no project work can proceed. Set it to `Done` only when the project
success criteria are complete or the issue records why the project ended.

A task remains `In Review` after its pull request is approved. Move all tasks in
the stack to `Done` only after the full stack lands and the task acceptance
criteria are confirmed against the default branch.

Prefer project views grouped or filtered by `Status` and type label. Do not add
lifecycle labels.

## Checkpoints

Add a checkpoint comment after planning or synthesis, before dispatching an
implementation batch, after a lane becomes ready, blocked, or cancelled, after
an integration batch is submitted, after review remediation changes stack SHAs,
and before ending a session:

```markdown
## Checkpoint

- **Status:** In Progress
- **Completed:** What changed since the previous checkpoint.
- **Learned:** Information that affects the plan.
- **Decisions:** Decisions made or still needed.
- **Validation:** Checks completed and unresolved findings.
- **Integration worktree:** The canonical integration worktree path.
- **Artifact root:** The project artifact root in the primary checkout.
- **Canonical stack:** The stack number, base SHA, and branch chain.
- **Pull requests:** Active reviews with immutable base and head SHAs.
- **Blockers:** None, or the blocker and owner.
- **Next action:** The exact next planning, lane, integration, or review action.

| Task | Phase | Worktree | Branch | Base/head | PR | Status | Next action |
|---|---|---|---|---|---|---|---|
| #123 | implementing | `/path` | `branch` | `abc/def` | none | active | Await handoff |
```

Update the parent issue's managed current state block to match the latest
checkpoint.

## Discovery

When no issue number is supplied:

1. Query the configured GitHub Project for open items with the `type: project`
   label.
2. Prefer an issue explicitly named by the user.
3. Otherwise prefer the most recently updated project whose `Status` is neither
   `Blocked` nor `Done`.
4. If more than one issue is a credible match, present the candidates rather
   than guessing.

## Reconciliation

Before selecting work, compare:

- The parent issue and latest checkpoint.
- Open and closed linked task issues.
- GitHub Project membership and `Status` values.
- Linked pull requests and their merge state.
- The GitHub stack order and approval state.
- Applicable decision records.
- The integration and staging worktrees, branch leases, working trees, expected
  bases, committed staging heads, and relevant commits.
- The recorded artifact root and referenced artifacts.

Issues record intent and durable context. The GitHub Project records lifecycle.
The repository records what has actually been implemented. Local artifacts hold
supporting evidence only. When they disagree, report the mismatch and update the
Project and checkpoint after resolving it. A missing local artifact must not
make the project impossible to reconstruct from GitHub.

## Worktree and stack setup

Create the integration worktree after the project brief and issue graph are
approved, before implementation starts:

1. Fetch the default branch and confirm the primary checkout does not contain
   project changes that need to move.
2. Create a sibling integration worktree at a stable path such as
   `../query.worktrees/project-123-integration`.
3. Create the bottom canonical stack branch in that worktree from the recorded
   default-branch base SHA. Use a descriptive branch name that includes the
   parent or task issue number.
4. Run `gh stack init <branch>` from the integration worktree. Always pass
   branch names, and configure `rerere.enabled` and `remote.pushDefault` first
   so the command remains non-interactive.
5. Create the project artifact root and record its absolute path, the integration
   worktree, canonical base SHA, and stack branch chain in the managed state and
   checkpoint.

For each implementation batch, select an antichain of actionable tasks that do
not depend on one another and have compatible integration contracts. Before
dispatch, record a lane manifest containing:

```text
Task issue
Expected base SHA
Worktree path
Staging branch
Expected files, modules, or interface boundary
Applicable instruction roots and skills
Acceptance criteria
Validation
Integration order and rationale
Agent role and model
Artifact directory and ownership
```

Create one staging branch and worktree per task from the same recorded batch
base or an already-stable foundational stack commit:

```text
../query.worktrees/project-123-task-456
project-123/task-456-work
```

Exactly one implementer owns each staging worktree and branch. Implementers may
create task-scoped conventional local commits but must not create or switch
branches, manage worktrees, run `gh stack`, or update GitHub.

After a lane completes, freeze its reported head SHA and verify its base,
commits, changed paths, instruction inventory, validation, and clean state.
Choose a deterministic topological integration order. For each lane:

1. Create the corresponding canonical stack branch in the integration worktree.
2. Cherry-pick the lane commits into that layer.
3. Validate the layer relative to its immediate stack base.
4. Continue with the next completed lane.
5. Run cross-task validation on the assembled canonical stack.

Staging branches do not receive pull requests. The canonical stack contains one
pull request per integrated task layer.

Plan stack layers in dependency order. A lower branch must contain everything a
higher branch needs. If implementation reveals that a lower layer must change,
serialize remediation through the integration worktree, commit the change in the
lowest affected layer, and run `gh stack rebase --upstack`. Do not send
concurrent implementers into separate layers of the canonical stack.

If replanning changes an active lane's scope, request cancellation and wait for
an acknowledged terminal handoff before releasing its slot. Inspect and preserve
recoverable work, then record whether the lane is abandoned, superseded,
blocked, or ready for a revised assignment.

## Pull request and review flow

Each canonical task branch has one pull request. A task is ready for review only
after its staging commits are integrated, the canonical layer and assembled
stack are validated, and the pull request is submitted and described:

1. Run `gh stack submit --auto` from the integration worktree to push the stack
   and create or update draft pull requests.
2. Fill out [the pull request template](../../../pull_request_template.md) for
   each integrated task. Generate the body at
   `.agents/tmp/projects/<project-key>/pr-bodies/<task-number>.md`. Include the parent
   project issue, close the task issue with a supported closing keyword, and
   describe the canonical behavior change, validation, and focused review
   notes. Do not duplicate stack metadata that `gh stack` already communicates.
3. Update each pull request with `gh pr edit --body-file <path>`. The stack
   extension generates titles and initial bodies, so the explicit edit is
   required.
4. Mark each submitted pull request ready with `gh pr ready <number>`.
5. Move the integrated tasks to `In Review`, record immutable base and head SHAs
   in the checkpoint, and fan out read-only reviewers within the five-slot
   limit.

Creating the pull request before delegating review lets the user and reviewer
inspect the same change at the same time. The reviewer is read-only and must not
modify the integration worktree or pull request. A review becomes stale when its
base or head SHA changes. Collect findings where practical, resolve them from the
lowest affected stack layer upward, rebase and resubmit upstack branches, update
every affected pull request body, and repeat materially invalidated reviews.

An approved pull request remains open while later stack layers are integrated.
After every committed task is approved and the project success criteria are
satisfied, synchronize the stack and use GitHub's atomic stack merge with
`gh stack merge <stack-number> --yes` and an explicit repository-appropriate
`--merge`, `--squash`, or `--rebase` flag. Do not use `gh pr merge` for project
stack pull requests. If any pull request cannot merge, none of the stack lands.

After the stack lands, run `gh stack sync --prune`, confirm the default branch
contains the project outcome, move the task issues and parent issue to `Done`,
and write the final checkpoint. Remove only clean staging and integration
worktrees whose commits are integrated or intentionally abandoned.

Clean up lane artifacts as soon as their evidence is consumed and essential
state is promoted. Clean up pull request body files after confirming GitHub has
the current body. At project completion, inspect and remove only the remaining
specific resolved project directory after confirming no pending integration,
review, or recovery action depends on it. Never remove the `.agents/tmp/` root
or use project-key globs.

## Execution rules

- Use at most five active sub-agents across all roles.
- Do not allow sub-agents to delegate further.
- Select only independent `Backlog` tasks with no open `blocked by`
  relationships for the same implementation batch.
- Move dispatched tasks to `In Progress` before implementation.
- Give every editing agent an exclusive staging worktree, branch, write lease,
  base SHA, and artifact directory.
- Require implementers to return task-scoped conventional commits and a clean
  staging worktree.
- Verify instruction coverage and required `mise` validation before integration.
- Serialize staging commits through the canonical integration worktree.
- Submit canonical pull requests and fill their bodies before moving tasks to
  `In Review`.
- Move issues to `Blocked` when an open native dependency prevents progress.
- Move unblocked issues back to `Backlog`.
- Run only one editing agent at a time in the canonical integration worktree.
- Fan out read-only reviews only against immutable base and head SHAs.
- Review before marking a task complete.
- Do not merge task pull requests individually.
- Do not close a task only because code exists or review is complete. Confirm
  its acceptance criteria and validation after the full stack lands.
- Re-plan when a lane reveals a false assumption, overlapping ownership,
  conflicting acceptance criteria, or a cross-task validation failure.
- Keep issue and pull request bodies current, and use comments as the
  chronological activity log.
- Clean up temporary artifacts after use and record why any artifact must remain.

## Required repository setup

Create these type labels if they do not exist:

- `type: project`
- `type: feature`
- `type: bug`
- `type: task`

Configure one GitHub Project and record its owner, number, and URL in
[config.md](./config.md). If project access is unavailable, stop and report the
missing authentication scope instead of falling back to lifecycle labels.

Native issue dependencies are available in personal and organization-owned
repositories. Prefer them over dependency text or dependency labels.

Install and authenticate the `github/gh-stack` GitHub CLI extension. The
repository must have GitHub stacked pull requests enabled. If `gh stack submit`
exits because stacks are unavailable, stop and report the missing repository
capability instead of creating unrelated pull requests.
