# Peak memory on macOS

## Table of contents

- [Measure a whole-project scan](#measure-a-whole-project-scan), L:9
- [Compare runs](#compare-runs), L:33
- [Profile the peak](#profile-the-peak), L:45

## Measure a whole-project scan

`workspace list --page 1` builds the project from the current directory before
paginating. It exercises discovery and parsing, not just the first page.

```sh
# From the query checkout
cargo build --workspace --release
QUERY="$PWD/target/release/query"

(
  cd /path/to/monorepo || exit 1
  for run in 1 2 3; do
    for jobs in 1 4 8; do
      printf '\nRun %s, %s Rayon threads\n' "$run" "$jobs"
      RAYON_NUM_THREADS="$jobs" /usr/bin/time -l \
        "$QUERY" workspace list --page 1 >/dev/null || exit 1
    done
  done
)
```

`/usr/bin/time -l` writes its measurements to stderr.

## Compare runs

Record `real` time (seconds) and `peak memory footprint` (bytes; divide by
1,048,576 for MiB). `maximum resident set size` is a separate metric. Report
the median and range for each thread count and note whether the runs used a
warm filesystem cache. A fresh process does not clear that cache.

Keep the input revision, machine, and build fixed. If memory rises with thread
count, try a smaller pool and check the runtime tradeoff. If it is already high
with one thread, inspect allocations before changing the pipeline. The
thread-count comparison does not identify which allocations caused the peak.

## Profile the peak

Use **Instruments → Allocations** to inspect live allocations and their call
stacks near the peak. Look for Oxc arena backing memory, owned parse results,
and project graph allocations.

In the current scan, [`Project::new`](../../../../crates/query/src/project.rs)
resets the Oxc arena after each parse and collects all owned `ParseResult`s
before discarding them. The benchmark measures discovery and parsing, not a
future linker or query index. If retained results dominate, batching helps
only if each batch is consumed or persisted rather than appended to the same
in-memory collection.
