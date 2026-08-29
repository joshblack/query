# Analysis contract

This contract defines the first-slice behavior independently of a parser,
resolver, graph library, or command-line interface.

## Domain boundaries

An analysis starts from one or more analysis roots. A module specifier is the
written reference in source, not a module identity. The resolver maps
`(importer, specifier, import kind)` to a logical module and source origin, an
external result, or a typed failure. Multiple specifiers may converge on one
logical module.

Each logical module owns its module facts, declarations, and source-backed
identities. Linking owns aliases, re-exports, final symbols, component edges,
route links, ambiguity, and cycle groups. A module fact is never owned by more
than one module.

## Provenance

Every source-backed fact includes provenance with a Source ID and a UTF-8 byte
span. Spans are zero-based and end-exclusive. One-based line and column
coordinates are included for display, but byte offsets are authoritative.
Source IDs describe source origins, while Module IDs describe logical modules.

The result must not contain absolute machine paths. Provenance may identify a
logical or repository-relative source origin, but it must not disclose local
machine layout.

## Completeness and diagnostics

Every result has one completeness state:

- `complete`: every requested root produced trustworthy findings, and no
  relevant condition limits them;
- `partial`: at least one requested root produced trustworthy findings, but a
  relevant unsupported, unresolved, ambiguous, or other non-fatal condition
  limits the result, or another requested root failed; or
- `failed`: no requested root produced trustworthy findings.

Every partial or failed result references diagnostics through
`reason_diagnostic_ids`. Diagnostic effect is one of `none`, `incomplete`, or
`fatal`; severity alone does not determine completeness. Diagnostics are
separate from findings.

Each requested root also has its own completeness state and
`reason_diagnostic_ids`. A fatal diagnostic fails its associated root. The
envelope is partial rather than failed when another root still has trustworthy
findings. Envelope reason IDs are the union of the limiting root reason IDs in
canonical diagnostic order.

Diagnostic categories are:

- `input`
- `source`
- `syntax`
- `unsupported`
- `resolution`
- `linkage`
- `conflict`
- `internal`

Expected errors are the conceptual mapping from these semantic categories to
user-facing failures and exit behavior. This document does not define an
implementation of `ExpectedError`.

Unresolved or unsupported dependencies do not erase independent trustworthy
findings. A failed result means that no trustworthy findings exist for the
requested roots, not merely that one dependency failed.

## Linking rules

An import alias targets the defining symbol when the imported binding resolves
to that declaration. Re-export chains retain their candidate identities until
linking can prove the final symbol. Ambiguous star re-exports retain a
deterministically sorted candidate list and make the result partial.

Route parsing records a candidate direct call. Linking recognizes a route only
when its callee resolves to a source-qualified symbol named `jsonRoute`.
The component expression must resolve to one source-qualified component
symbol. Shadowed, unresolved, unsupported aliased, member, indirect, and
computed calls have explicit candidate outcomes. A candidate with an
unresolved or ambiguous component is not a recognized route. Duplicate route
declarations are preserved and reported as ambiguity, never selected by
traversal order.

## Cycles and traversal

Module and component cycles are normal graph structure and do not by
themselves make a result incomplete. Cycle groups are deterministic strongly
connected groups for a self-loop or for multiple nodes.

Traversal terminates, uses minimum distance, excludes the starting component
from recursive ancestor and descendant results, and preserves direct self-edges.
Reverse component relationships are derived from canonical render-to-render
edges rather than duplicated in the result.

## Deterministic ordering

All ordering is ascending unless stated otherwise:

- roots by Root ID;
- sources by Source ID;
- modules by Module ID;
- findings by `kind`, subject ID, provenance start byte, and Finding ID;
- diagnostics by Source ID, with diagnostics without a Source ID last, then
  start byte, code, subject ID, and Diagnostic ID;
- provenance entries by Source ID, start byte, and end byte;
- cycle members, ambiguity candidates, and reason diagnostic IDs by their
  referenced stable ID;
- diagnostic Root IDs by Root ID;
- import and export bindings by local or exported name, binding kind,
  provenance start byte, and Finding ID; and
- traversal results by minimum distance, Symbol ID, and supporting Finding ID.

Paths used as sort keys use `/` separators and Unicode scalar value ordering
without locale-sensitive comparison. Ordering must not depend on filesystem
enumeration, hash map iteration, parser traversal, graph indices, or task
completion order.

## Versioned result envelope

The JSON envelope is named `query-analysis` and has version `1`. It contains
`completeness`, `roots`, `sources`, `modules`, `findings`, and `diagnostics`.
IDs are opaque and deterministic for byte-identical inputs with the same
analysis root and resolver configuration. They are not promised to remain
stable across edits, renames, root changes, or policy changes.

The envelope uses explicitly tagged object shapes. It does not expose OXC
types, petgraph indices, random UUIDs, counters, or implementation-specific
objects. The complete field-level shape is specified in
[JSON format v1](json-format-v1.md).

## Out of scope

The first slice does not define parser, resolver, graph, or CLI
implementation; package resolution; CommonJS; persistence; concurrency; or a
general framework plugin API.
