# Query analysis context

This glossary establishes the terms used to describe JavaScript and TypeScript
module analysis. The terms are intentionally distinct where they describe
different identities, ownership, or relationships.

## Analysis vocabulary

**Analysis root**:
The stable path base used to identify requested sources and normalize
file-backed source origins.

**Module specifier**:
The written string used to refer to a module from an import or export.

**Logical module**:
The module identity that represents one analyzed module, independent of where
its source was found or how it was written by an importer.

**Module ID**:
The opaque identifier of a logical module.

**Source**:
The UTF-8 text associated with a module.

**Source origin**:
The origin from which a source was obtained.

**Source ID**:
The opaque identifier of a source origin.

**Provenance**:
The source location that supports an analyzed fact.

**Module fact**:
A fact that belongs to exactly one logical module.

**Declaration**:
A source construct that introduces a name or an analyzable entity.

**Symbol**:
A declaration or other named entity that can be referenced.

**Symbol ID**:
The opaque identifier of a symbol.

**Export binding**:
A name made available outside its declaring module.

**Alias**:
A reference that gives an existing symbol another name.

**Re-export**:
An export binding that exposes a symbol from another module.

**Component**:
A renderable PascalCase function or function-valued variable.

**Component edge**:
A rendering relationship from the component that renders to the component
that is rendered.

**Route declaration**:
A recognized route with a literal path and a component target.

**Finding**:
A trustworthy analyzed result exposed to a consumer.

**Diagnostic**:
A structured explanation of an input, source, syntax, support, resolution,
linkage, conflict, or internal condition.

**Completeness**:
The result state describing whether the requested analysis is complete,
partial, or failed.

**Cycle group**:
A deterministic group of modules or components that are mutually reachable,
including a group containing one node with a self-loop.
