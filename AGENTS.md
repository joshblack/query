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

### General

- Model the full error space—no shortcuts or simplified error handling.
- Handle all edge cases, including race conditions, signal timing, and platform differences.
- Use the type system to encode correctness constraints.
- Prefer compile-time guarantees over runtime checks where possible.
- Do not create, update, upvote, or downvote Copilot memories. Keep durable
  project knowledge in tracked documentation, GitHub issues, pull requests, and
  project checkpoints.

#### User experience as a primary driver

- Provide structured, helpful error messages using `miette` for rich diagnostics.
- Make progress reporting responsive and informative.
- Maintain consistency across platforms even when underlying OS capabilities differ. Use OS-native logic rather than trying to emulate Unix on Windows (or vice versa).
- Write user-facing messages in clear, present tense: "query now supports..." not "query now supported..."

#### Pragmatic incrementalism

- Prefer specific, composable logic over abstract frameworks.
- Evolve the design incrementally rather than attempting perfect upfront architecture.
- Document design decisions and trade-offs in design docs
- When uncertain, explore and iterate

#### Production-grade engineering

- Use type system extensively: newtypes, builder patterns, type states, lifetimes.
- Use message passing or the actor model to avoid data races.
- Test comprehensively, including edge cases, race conditions, and stress tests.
- Pay attention to what facilities already exist for testing, and aim to reuse them.
- Getting the details right is really important!

#### Documentation

- Use inline comments to explain "why," not just "what".
- Don't add narrative comments in function bodies. Only add a comment if what you're doing is non-obvious or special in some way, or if something needs a deeper "why" explanation.
- Module-level documentation should explain purpose and responsibilities.
- **Always** use periods at the end of code comments.
- **Never** use title case in headings and titles. Always use sentence case.
- Always use the Oxford comma.
- Don't omit articles ("a", "an", "the"). Write "the file has a newer version" not "file has newer version".

### Code style

### Type system patterns

- **Newtypes** for domain types
- **Builder patterns** for complex construction
- **Type states** encoded in generics when state transitions matter
- **Lifetimes** used extensively to avoid cloning
- **Restricted visibility**: Use `pub(crate)` and `pub(super)` liberally
- **Non-exhaustive in stable crates**: The `query` crate has a stable API and public types there should be `#[non_exhaustive]` for forward compatibility. Internal crates do not have stable APIs, so `#[non_exhaustive]` is not required (though error types may still use it).

### Error handling

- Use `thiserror` for error types with `#[derive(Error)]`.
- Group errors by category with an `ErrorKind` enum when appropriate.
- Provide rich error context using structured error types.
  - Parts of the code use `miette` for structured error handling.
- Two-tier error model:
  - `ExpectedError`: User/external errors with semantic exit codes.
  - Internal errors: Programming errors that may panic or use internal error types.
- Error display messages should be lowercase sentence fragments suitable for "failed to {error}".

### Serde patterns

- Use `serde_ignored` for ignored paths in configuration.
- Never use `#[serde(flatten)]`. Instead, copy fields to structs as necessary. The internal buffering leads to poor warnings from `serde_ignored`.
- Never use `#[serde(untagged)]` for deserializers, since it produces poor error messages. Instead, write custom visitors with an appropriate `expecting` method.

### Serialization format changes

When modifying any struct that is serialized to disk or over the wire:

1. **Trace the full version matrix**:
   - Old reader + new data: Can it deserialize? Does it lose information?
   - New reader + old data: Does `#[serde(default)]` produce correct values?
   - Old writer + new data: Can it round-trip without data loss? (This is the easy one to miss!)

2. **Bump format versions proactively**: If adding a field that will be semantically important, bump the version when adding the field, not when first using non-default values. This prevents older versions from silently corrupting data on write-back.

3. **`#[serde(default)]` is necessary but not sufficient**: It allows old readers to deserialize new data, but old writers will still drop unknown fields on write-back.

### Async patterns

- Use `tokio` for async runtime (multi-threaded).
- Be selective with async. Only use it in runner and runner-adjacent code.
- Use async for I/O and concurrency, keep other code synchronous.
- Use `async-scoped` for structured concurrency without `'static` bounds.
- Use `future-queue` for backpressure-aware task scheduling.

### Module organization

- Use `mod.rs` files to re-export public items.
- Do not put any nontrivial logic in `mod.rs` -- instead, it should go in a more specific submodule.
- Keep module boundaries strict with restricted visibility.
- Platform-specific code in separate files: `unix.rs`, `windows.rs`.
- Use `#[cfg(unix)]` and `#[cfg(windows)]` for conditional compilation.
- Test helpers in dedicated modules/files.
- Use fully qualified imports rarely, prefer importing the type most of the time, or otherwise a module if it is conventional.
- Never write `std::fmt::Display` as a fully qualified type. Instead, import `std::fmt` and use `fmt::Display`.
- **Always** import types or functions at the very top of the module, with the one exception being `cfg()`-gated functions. Never import types or modules within function contexts, other than this `cfg()`-gated exception.
- It is okay to import enum variants for pattern matching

### Memory and performance

- Use `Arc` or borrows for shared immutable data.
- Careful attention to cloning referencing. Avoid cloning if code has a natural tree structure.
- Stream data (e.g. iterators) where possible rather than buffering.

### String formatting

- The `clippy::format_push_string` lint is enabled. If triggered, use the `swrite!` macro from the `swrite` crate instead of `push_str(&format!(...))`.

### Lint attributes

- Always use `#[expect(...)]` instead of `#[allow(...)]` for suppressing lints. The `expect` attribute will warn if the lint is no longer triggered, helping to keep the codebase clean.

### Testing

#### Test organization

- Unit tests in the same file as the code they test.
- Integration tests in `tests/`

#### Testing tools

- **test-case**: For parameterized tests.
- **insta**: For snapshot testing.
- **pretty_assertions**: For better assertion output.

### Commits

#### Message style

- Always use conventional commits.
- Keep descriptions concise but descriptive.
- Use simple past and present tense: "Previously, when the user did X, Y used to happen. With this commit, now Z happens. Also add tests for U, V, and W."
- Commit messages should be Markdown. Don't use backticks in commit message titles, but do use them in bodies.

#### Commit quality

- **Atomic commits**: Each commit should be a logical unit of change.
- **Bisect-able history**: Every commit must build and pass all checks.
- **Separate concerns**: Format fixes and refactoring should be in separate commits from feature changes.

### GitHub Actions

- Pin actions to full commit SHAs and include the corresponding full semantic
  version in a comment, e.g. prefer `foo/bar@{sha} # v1.2.3` vs `foo/bar@v1`.

## Documentation

- Docs live in `/docs`.
- Architecture decision records (ADRs) live in `/docs/adrs`
- Use the template in `/docs/adrs/YYYY-MM-DD-template.md` for new decisions
- Use ADRs to capture the following types of decisions:
  - General coding guidelines or practices that affect the entire project
  - Specific architecture guidelines or practices that will influence or be used
    for upcoming work
- Domain knowledge lives in `CONTEXT.md` files. These files:
  - Document domain or language concepts for a particular part of the project
  - Borrow from domain-driven design, in particular ubiquitous language and
    bounded contexts, to have a shared vocabulary between agents and humans
  - Contain flow or mermaid charts to help with communicating domain objects,
    relationships, or broader concepts
- Whenever a change is made that may impact documentation above, make sure to
  either update it directly or leave a note about what has changed or superceded
  it (e.g. note if an ADR supercedes another one or if an ADR is superceded by
  another one)

## Worktrees

- Store worktrees under the sibling `query.worktrees/` directory.
- Use one descriptive subdirectory per project or task, and perform project work
  from that worktree.

## Self improvement

- If the user asks you to make changes to an implementation, see if there is a
  way to update @AGENTS.md or create/update a skill, sub-agent, etc so that this
  does not need to happen in the future
- Ask the user to confirm before updating these resources with a change
- If resources are contradictory or content is superfluous suggest making a
  change to simplify
- Always provide the minimal set of instructions to satisfy the request or
  communicate the convention
