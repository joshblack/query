---
name: Update mise tools
description: Open a pull request pinning mise tools to their latest stable releases.
on:
  schedule:
    - cron: "0 13 * * 1"
  workflow_dispatch:
permissions:
  contents: read
  pull-requests: read
  copilot-requests: write
engine: copilot
timeout-minutes: 15
network:
  allowed:
    - defaults
    - github
tools:
  edit:
  bash:
    - "cat:*"
    - "git diff:*"
    - "git status:*"
  github:
    toolsets:
      - pull_requests
safe-outputs:
  create-pull-request:
    allowed-files:
      - mise.toml
    max-patch-files: 1
    title-prefix: "chore(deps): "
    draft: false
    max: 1
    fallback-as-issue: false
steps:
  - name: Install workflow script dependencies
    run: npm install --prefix /tmp/gh-aw/agent/mise-tools --no-save --package-lock=false --ignore-scripts smol-toml@1.9.1
  - name: Set up mise without installing project tools
    uses: jdx/mise-action@236a21c3b91f5db6d4e174b348674a0f0caac06d # v5.2.9
    with:
      install: false
      cache: false
  - name: Resolve latest stable tool versions
    run: |
      node <<'JS'
      const { execFileSync } = require('node:child_process');
      const { mkdirSync, readFileSync, writeFileSync } = require('node:fs');
      const { parse } = require('/tmp/gh-aw/agent/mise-tools/node_modules/smol-toml');

      const original = readFileSync('mise.toml', 'utf8');
      const config = parse(original);
      const versions = {};
      for (const [tool, entry] of Object.entries(config.tools)) {
        const current = typeof entry === 'string' ? entry : entry.version;
        const latest = execFileSync('mise', ['latest', tool], {
          encoding: 'utf8',
          timeout: 120_000,
        }).trim();
        if (!/^\d+\.\d+\.\d+$/.test(latest)) {
          throw new Error(`${tool}: expected a stable release, got ${latest}`);
        }
        versions[tool] = {
          current,
          latest,
        };
        console.log(`${tool}: ${current} -> ${latest}`);
      }

      const directory = '/tmp/gh-aw/agent';
      mkdirSync(directory, {
        recursive: true,
      });
      writeFileSync(`${directory}/mise-original.toml`, original);
      writeFileSync(`${directory}/mise-versions.json`, `${JSON.stringify(versions, null, 2)}\n`);
      JS
post-steps:
  - name: Verify only tool versions change
    if: success()
    run: |
      node <<'JS'
      const { readFileSync } = require('node:fs');
      const { isDeepStrictEqual } = require('node:util');
      const { parse } = require('/tmp/gh-aw/agent/mise-tools/node_modules/smol-toml');

      const directory = '/tmp/gh-aw/agent';
      const before = parse(readFileSync(`${directory}/mise-original.toml`, 'utf8'));
      const after = parse(readFileSync('mise.toml', 'utf8'));
      if (!isDeepStrictEqual(before, after)) {
        const versions = JSON.parse(readFileSync(`${directory}/mise-versions.json`, 'utf8'));
        for (const [tool, resolved] of Object.entries(versions)) {
          const entry = before.tools[tool];
          if (typeof entry === 'string') {
            before.tools[tool] = resolved.latest;
          } else {
            entry.version = resolved.latest;
          }
        }
        if (!isDeepStrictEqual(before, after)) {
          throw new Error('Only the resolved tool version updates are allowed');
        }
      }
      JS
---

# Update mise tools

Read `mise.toml` and `/tmp/gh-aw/agent/mise-versions.json`. The report contains
fresh versions resolved by `mise latest` for each entry in `[tools]`.

Before editing, check for an open pull request created by this workflow with
the title `chore(deps): update mise tools`. If one exists, use the noop safe
output to explain that it should be reviewed first. Do not create a duplicate.

Update every `[tools]` entry to the exact `latest` version in the report.
Include major upgrades and replace floating versions such as `latest` or
`12` with exact releases. In particular, pin `mr-boxington` instead of keeping
it on `latest`.

Change only version strings in `mise.toml`. Preserve all other content and
formatting, including `[settings]`, `[wrappers.cargo]`, tasks, `postinstall`,
`lazy`, and `lazy_bins`. Do not update `rust-toolchain.toml`, install tools,
run postinstall hooks, or modify any other file. If all versions are already
exactly current, use the noop safe output and do not create a pull request.

Review `git diff --check` and `git diff -- mise.toml` before requesting the
create-pull-request safe output. Do not commit changes locally; leave the
version-only diff for the safe-output handler.

Use the pull request title `update mise tools` (the configured prefix adds
`chore(deps): `). Fill out the repository's pull request template if one
exists, leaving issue references blank when there is no issue. Include a short
table of tool names, previous versions, and new versions. Cite `mise latest
<tool>` as the source of each resolved release and call out major upgrades.
Do not include local environment notes or standard CI checks in the body.
