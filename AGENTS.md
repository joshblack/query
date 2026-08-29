# AGENTS.md

## Project setup

This project uses [mise](https://mise.jdx.dev/) to install the Rust toolchain
and Cargo utilities declared in `mise.toml`.

From the repository root, initialize the environment with:

```sh
mise trust
mise install
```

If mise is not activated in the current shell, run commands through
`mise exec`:

```sh
mise exec -- cargo check --workspace
```

When available, mise uses `cargo-binstall` to install Cargo utilities such as
`cargo-nextest` and `cargo-shear`.

## Validation

Use the tasks defined in `mise.toml` to validate changes:

```sh
mise run fmt
mise run lint
mise run test
```

Run the checks relevant to the work before finishing. Prefer checking only the
changed files, targets, or tests when the underlying command supports that
without changing the workspace feature resolution. For example, format-check
specific Rust files with:

```sh
mise run fmt -- -- src/main.rs
```

When a task does not support safe, focused validation, run the full task.

## Conventions

### Rust

### GitHub Actions

- Pin actions to full commit SHAs and include the corresponding full semantic
  version in a comment, e.g. prefer `foo/bar@{sha} # v1.2.3` vs `foo/bar@v1`.

## Documentation

- Docs live in `/docs`.
- Decision records live in `/docs/decisions`
- Use the template in `/docs/docs/YYYY-MM-DD-template.md` for new decisions

## Planning

- Use the project's issue tracker in GitHub for managing work as issues
- Organize work into broad and narrow issues as needed. Use sub-issues as
  appropriate for work that goes into a broader project or goal issue
- Use issue relationships to communicate if work is blocked by another issue
- Write the minimal description possible in an issue to convey the point
- Communicate the background (if any) and what work the issue is tracking

## Self improvement

- If the user asks you to make changes to an implementation, see if there is a
  way to update @AGENTS.md or create/update a skill, sub-agent, etc so that this
  does not need to happen in the future
- Ask the user to confirm before updating these resources with a change
- If resources are contradictory or content is superfluous suggest making a
  change to simplify
- Always provide the minimal set of instructions to satisfy the request or
  communicate the convention
