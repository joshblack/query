---
name: benchmark
description: Use when measuring Query's runtime or peak memory on representative projects, comparing Rayon thread counts, or investigating what drives a memory peak. Do not use for correctness checks or unrelated performance work.
---

# Benchmark

- A **measurement** reports one run. A **benchmark** compares repeated runs
  under controlled conditions. A **profile** explains where time or memory goes.
- Build once in release mode. Measure the binary directly on the same checkout
  of the input project. Change one setting at a time and record the build,
  workload, machine, and settings.
- Report wall time and peak memory together. Profile only when you need to
  explain the result.

## References

| Reference | When to use |
| --- | --- |
| [Peak memory on macOS](./references/peak-memory-macos.md) | Measure a whole-project scan, compare Rayon thread counts, or investigate what contributes to the peak. |
