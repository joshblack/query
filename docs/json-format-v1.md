# JSON format v1

This document specifies the versioned result shape for the first analysis
slice. Every variant uses an explicit `kind` field.

## Envelope

```json
{
  "name": "query-analysis",
  "version": 1,
  "analysis_root": ".",
  "completeness": {
    "state": "complete",
    "reason_diagnostic_ids": []
  },
  "roots": [],
  "sources": [],
  "modules": [],
  "findings": [],
  "diagnostics": []
}
```

The required fields are `name`, `version`, `analysis_root`, `completeness`,
`roots`, `sources`, `modules`, `findings`, and `diagnostics`. `name` is
`query-analysis`, and `version` is `1`.

`analysis_root` is the normalized path base for file-backed origins. It uses
`/` separators and does not contain an absolute machine path.

## Identity

Root IDs, Source IDs, Module IDs, Symbol IDs, Finding IDs, and Diagnostic IDs
are opaque strings. They are deterministic for byte-identical inputs under the
same analysis root and resolver configuration. They do not promise stability
across edits, renames, root changes, or resolver policy changes.

IDs do not contain random UUIDs, process-local counters, absolute machine
paths, OXC identities, or petgraph indices. Consumers compare complete IDs and
do not parse their spelling.

## Roots and completeness

Each requested root has its own status:

```json
{
  "root_id": "root:src/routes.tsx",
  "module_id": "module:src/routes.tsx",
  "state": "complete",
  "reason_diagnostic_ids": []
}
```

`module_id` is absent when resolution cannot identify a logical module.

The envelope status is:

- `complete` when every root is complete;
- `partial` when at least one root has trustworthy findings and any root is
  partial or failed; or
- `failed` when no root has trustworthy findings.

Every partial or failed root lists the diagnostics that limit it. Envelope
`reason_diagnostic_ids` is the union of those IDs in canonical diagnostic
order.

## Sources and modules

A source describes acquired text and its origin:

```json
{
  "source_id": "source:src/button.tsx",
  "language": "tsx",
  "origin": {
    "kind": "file",
    "path": "src/button.tsx"
  }
}
```

A module record connects a logical module to its source:

```json
{
  "module_id": "module:src/button.tsx",
  "source_id": "source:src/button.tsx"
}
```

Sources do not contain module specifiers. A written specifier belongs to the
import or re-export finding where it appears. Several specifiers can resolve
to one Module ID without duplicating a source or module record.

## Provenance

Every source-backed finding has a non-empty `provenance` array:

```json
[
  {
    "source_id": "source:src/routes.tsx",
    "span": {
      "start_byte": 84,
      "end_byte": 92,
      "start": {
        "line": 4,
        "column": 11
      },
      "end": {
        "line": 4,
        "column": 19
      }
    }
  }
]
```

Byte offsets are zero-based UTF-8 offsets, and the end is exclusive. Lines and
columns are one-based display coordinates. Byte offsets are authoritative.

Derived findings, including cycle groups and ambiguities, contain the
provenance of the source-backed findings that support them. A diagnostic may
have an empty provenance array only when no source was acquired, such as an
invalid analysis root or an unavailable source.

## Finding shapes

Every finding has a `finding_id`, a `kind`, and non-empty `provenance`.

### Symbols

```json
{
  "finding_id": "finding:symbol:button",
  "kind": "symbol",
  "symbol_id": "symbol:src/button.tsx:Button",
  "module_id": "module:src/button.tsx",
  "name": "Button",
  "namespace": "value",
  "symbol_kind": "component",
  "provenance": [
    {
      "source_id": "source:src/button.tsx",
      "span": {
        "start_byte": 7,
        "end_byte": 13,
        "start": {
          "line": 1,
          "column": 8
        },
        "end": {
          "line": 1,
          "column": 14
        }
      }
    }
  ]
}
```

### Imports

```json
{
  "finding_id": "finding:import:app-button",
  "kind": "import",
  "module_id": "module:src/app.tsx",
  "specifier": "./button",
  "import_kind": "named",
  "imported_name": "Button",
  "local_name": "PrimaryButton",
  "resolution": {
    "kind": "resolved",
    "target_module_id": "module:src/button.tsx",
    "target_symbol_id": "symbol:src/button.tsx:Button"
  },
  "provenance": [
    {
      "source_id": "source:src/app.tsx",
      "span": {
        "start_byte": 0,
        "end_byte": 43,
        "start": {
          "line": 1,
          "column": 1
        },
        "end": {
          "line": 1,
          "column": 44
        }
      }
    }
  ]
}
```

Import resolution variants are `resolved`, `external`, `unresolved`, and
`ambiguous`. Ambiguous resolution contains sorted `candidate_symbol_ids`.

### Exports and re-exports

```json
{
  "finding_id": "finding:export:primary-button",
  "kind": "export",
  "module_id": "module:src/index.ts",
  "exported_name": "PrimaryButton",
  "export_kind": "named_reexport",
  "specifier": "./button",
  "resolution": {
    "kind": "resolved",
    "target_symbol_id": "symbol:src/button.tsx:Button",
    "via_export_finding_ids": [
      "finding:export:button"
    ]
  },
  "provenance": [
    {
      "source_id": "source:src/index.ts",
      "span": {
        "start_byte": 0,
        "end_byte": 52,
        "start": {
          "line": 1,
          "column": 1
        },
        "end": {
          "line": 1,
          "column": 53
        }
      }
    }
  ]
}
```

Local exports omit `specifier`. Export resolution variants are `resolved`,
`unresolved`, and `ambiguous`. Ambiguous resolution contains sorted candidate
Symbol IDs. Re-export chains contain ordered Finding IDs from the exposed
binding toward the defining symbol.

### Component edges

```json
{
  "finding_id": "finding:component-edge:app-button",
  "kind": "component_edge",
  "from_symbol_id": "symbol:src/app.tsx:App",
  "to_symbol_id": "symbol:src/button.tsx:Button",
  "provenance": [
    {
      "source_id": "source:src/app.tsx",
      "span": {
        "start_byte": 74,
        "end_byte": 91,
        "start": {
          "line": 4,
          "column": 10
        },
        "end": {
          "line": 4,
          "column": 27
        }
      }
    }
  ]
}
```

The canonical direction is from the component that statically renders to the
component that is rendered. Reverse relationships are derived.

### Route candidates and routes

Parsing can preserve a candidate that linking cannot recognize:

```json
{
  "finding_id": "finding:route-candidate:settings",
  "kind": "route_candidate",
  "path": "/settings",
  "callee": {
    "kind": "resolved",
    "symbol_id": "symbol:src/routes.tsx:jsonRoute"
  },
  "component": {
    "kind": "unresolved",
    "name": "Settings"
  },
  "provenance": [
    {
      "source_id": "source:src/routes.tsx",
      "span": {
        "start_byte": 20,
        "end_byte": 79,
        "start": {
          "line": 2,
          "column": 1
        },
        "end": {
          "line": 5,
          "column": 3
        }
      }
    }
  ]
}
```

A recognized route requires both the callee and component to resolve:

```json
{
  "finding_id": "finding:route:settings",
  "kind": "route",
  "path": "/settings",
  "callee_symbol_id": "symbol:src/routes.tsx:jsonRoute",
  "component_symbol_id": "symbol:src/settings.tsx:Settings",
  "candidate_finding_id": "finding:route-candidate:settings",
  "provenance": [
    {
      "source_id": "source:src/routes.tsx",
      "span": {
        "start_byte": 20,
        "end_byte": 79,
        "start": {
          "line": 2,
          "column": 1
        },
        "end": {
          "line": 5,
          "column": 3
        }
      }
    }
  ]
}
```

The callee must resolve to a source-qualified symbol named `jsonRoute`, and the
component must resolve to one source-qualified component symbol. An unresolved
or ambiguous candidate is not a recognized route.

### Ambiguities

```json
{
  "finding_id": "finding:ambiguity:settings-routes",
  "kind": "ambiguity",
  "ambiguity_kind": "duplicate_route",
  "subject": {
    "kind": "route_path",
    "path": "/settings"
  },
  "candidate_finding_ids": [
    "finding:route:settings-a",
    "finding:route:settings-b"
  ],
  "provenance": [
    {
      "source_id": "source:src/routes.tsx",
      "span": {
        "start_byte": 20,
        "end_byte": 138,
        "start": {
          "line": 2,
          "column": 1
        },
        "end": {
          "line": 8,
          "column": 3
        }
      }
    }
  ]
}
```

Ambiguity candidates are retained and sorted. An ambiguity never selects a
candidate by discovery or traversal order.

### Cycle groups

```json
{
  "finding_id": "finding:component-cycle:a-b",
  "kind": "cycle_group",
  "cycle_kind": "component",
  "member_ids": [
    "symbol:src/a.tsx:A",
    "symbol:src/b.tsx:B"
  ],
  "supporting_finding_ids": [
    "finding:component-edge:a-b",
    "finding:component-edge:b-a"
  ],
  "provenance": [
    {
      "source_id": "source:src/a.tsx",
      "span": {
        "start_byte": 30,
        "end_byte": 35,
        "start": {
          "line": 2,
          "column": 10
        },
        "end": {
          "line": 2,
          "column": 15
        }
      }
    },
    {
      "source_id": "source:src/b.tsx",
      "span": {
        "start_byte": 30,
        "end_byte": 35,
        "start": {
          "line": 2,
          "column": 10
        },
        "end": {
          "line": 2,
          "column": 15
        }
      }
    }
  ]
}
```

Members and supporting Finding IDs are sorted. A group contains multiple
members or one member with a direct self-edge.

## Diagnostics

```json
{
  "diagnostic_id": "diagnostic:unresolved-settings",
  "code": "unresolved_symbol",
  "category": "linkage",
  "effect": "incomplete",
  "severity": "error",
  "root_ids": [
    "root:src/routes.tsx"
  ],
  "subject": {
    "kind": "finding",
    "finding_id": "finding:route-candidate:settings"
  },
  "message": "the route component does not resolve to one symbol",
  "provenance": [
    {
      "source_id": "source:src/routes.tsx",
      "span": {
        "start_byte": 62,
        "end_byte": 70,
        "start": {
          "line": 4,
          "column": 14
        },
        "end": {
          "line": 4,
          "column": 22
        }
      }
    }
  ]
}
```

Categories are `input`, `source`, `syntax`, `unsupported`, `resolution`,
`linkage`, `conflict`, and `internal`. Effects are `none`, `incomplete`, and
`fatal`. Effect, not severity, controls completeness. `root_ids` scopes the
diagnostic to requested roots. An internal diagnostic is fatal and is not an
expected error.

## Deterministic ordering

All ordering is ascending:

- roots by Root ID;
- sources by Source ID;
- modules by Module ID;
- findings by `kind`, subject ID, provenance start byte, and Finding ID;
- diagnostics by Source ID, with diagnostics without a Source ID last, then
  start byte, code, subject ID, and Diagnostic ID;
- provenance by Source ID, start byte, and end byte;
- cycle members, candidate IDs, and reason diagnostic IDs by stable ID;
- diagnostic Root IDs by Root ID; and
- traversal results by minimum distance, Symbol ID, and supporting Finding ID.

Paths use `/` separators and Unicode scalar value ordering without
locale-sensitive comparison.

## Representative complete result

This result contains a relative import alias, a named re-export, a component
edge, and a recognized route. Provenance uses the shape defined above.

```json
{
  "name": "query-analysis",
  "version": 1,
  "analysis_root": ".",
  "completeness": {
    "state": "complete",
    "reason_diagnostic_ids": []
  },
  "roots": [
    {
      "root_id": "root:src/routes.tsx",
      "module_id": "module:src/routes.tsx",
      "state": "complete",
      "reason_diagnostic_ids": []
    }
  ],
  "sources": [
    {
      "source_id": "source:src/button.tsx",
      "language": "tsx",
      "origin": {
        "kind": "file",
        "path": "src/button.tsx"
      }
    },
    {
      "source_id": "source:src/routes.tsx",
      "language": "tsx",
      "origin": {
        "kind": "file",
        "path": "src/routes.tsx"
      }
    }
  ],
  "modules": [
    {
      "module_id": "module:src/button.tsx",
      "source_id": "source:src/button.tsx"
    },
    {
      "module_id": "module:src/routes.tsx",
      "source_id": "source:src/routes.tsx"
    }
  ],
  "findings": [
    {
      "finding_id": "finding:component-edge:routes-button",
      "kind": "component_edge",
      "from_symbol_id": "symbol:src/routes.tsx:App",
      "to_symbol_id": "symbol:src/button.tsx:Button",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 120,
            "end_byte": 137,
            "start": {
              "line": 6,
              "column": 10
            },
            "end": {
              "line": 6,
              "column": 27
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:export:primary-button",
      "kind": "export",
      "module_id": "module:src/routes.tsx",
      "exported_name": "PrimaryButton",
      "export_kind": "named_reexport",
      "specifier": "./button",
      "resolution": {
        "kind": "resolved",
        "target_symbol_id": "symbol:src/button.tsx:Button",
        "via_export_finding_ids": []
      },
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 43,
            "end_byte": 91,
            "start": {
              "line": 2,
              "column": 1
            },
            "end": {
              "line": 2,
              "column": 49
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:import:primary-button",
      "kind": "import",
      "module_id": "module:src/routes.tsx",
      "specifier": "./button",
      "import_kind": "named",
      "imported_name": "Button",
      "local_name": "PrimaryButton",
      "resolution": {
        "kind": "resolved",
        "target_module_id": "module:src/button.tsx",
        "target_symbol_id": "symbol:src/button.tsx:Button"
      },
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 0,
            "end_byte": 42,
            "start": {
              "line": 1,
              "column": 1
            },
            "end": {
              "line": 1,
              "column": 43
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:route:settings",
      "kind": "route",
      "path": "/settings",
      "callee_symbol_id": "symbol:src/routes.tsx:jsonRoute",
      "component_symbol_id": "symbol:src/button.tsx:Button",
      "candidate_finding_id": "finding:route-candidate:settings",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 138,
            "end_byte": 196,
            "start": {
              "line": 7,
              "column": 1
            },
            "end": {
              "line": 10,
              "column": 3
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:route-candidate:settings",
      "kind": "route_candidate",
      "path": "/settings",
      "callee": {
        "kind": "resolved",
        "symbol_id": "symbol:src/routes.tsx:jsonRoute"
      },
      "component": {
        "kind": "resolved",
        "symbol_id": "symbol:src/button.tsx:Button"
      },
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 138,
            "end_byte": 196,
            "start": {
              "line": 7,
              "column": 1
            },
            "end": {
              "line": 10,
              "column": 3
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:symbol:app",
      "kind": "symbol",
      "symbol_id": "symbol:src/routes.tsx:App",
      "module_id": "module:src/routes.tsx",
      "name": "App",
      "namespace": "value",
      "symbol_kind": "component",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 92,
            "end_byte": 95,
            "start": {
              "line": 4,
              "column": 10
            },
            "end": {
              "line": 4,
              "column": 13
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:symbol:button",
      "kind": "symbol",
      "symbol_id": "symbol:src/button.tsx:Button",
      "module_id": "module:src/button.tsx",
      "name": "Button",
      "namespace": "value",
      "symbol_kind": "component",
      "provenance": [
        {
          "source_id": "source:src/button.tsx",
          "span": {
            "start_byte": 7,
            "end_byte": 13,
            "start": {
              "line": 1,
              "column": 8
            },
            "end": {
              "line": 1,
              "column": 14
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:symbol:json-route",
      "kind": "symbol",
      "symbol_id": "symbol:src/routes.tsx:jsonRoute",
      "module_id": "module:src/routes.tsx",
      "name": "jsonRoute",
      "namespace": "value",
      "symbol_kind": "function",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 197,
            "end_byte": 206,
            "start": {
              "line": 12,
              "column": 10
            },
            "end": {
              "line": 12,
              "column": 19
            }
          }
        }
      ]
    }
  ],
  "diagnostics": []
}
```

The import and re-export specifiers both resolve to the same logical module.
Other spellings, such as `./button.tsx`, can resolve to that Module ID without
changing the source or module records.

## Representative partial result

This result retains an independent symbol while a route candidate has an
unresolved component.

```json
{
  "name": "query-analysis",
  "version": 1,
  "analysis_root": ".",
  "completeness": {
    "state": "partial",
    "reason_diagnostic_ids": [
      "diagnostic:unresolved-settings"
    ]
  },
  "roots": [
    {
      "root_id": "root:src/routes.tsx",
      "module_id": "module:src/routes.tsx",
      "state": "partial",
      "reason_diagnostic_ids": [
        "diagnostic:unresolved-settings"
      ]
    }
  ],
  "sources": [
    {
      "source_id": "source:src/routes.tsx",
      "language": "tsx",
      "origin": {
        "kind": "file",
        "path": "src/routes.tsx"
      }
    }
  ],
  "modules": [
    {
      "module_id": "module:src/routes.tsx",
      "source_id": "source:src/routes.tsx"
    }
  ],
  "findings": [
    {
      "finding_id": "finding:route-candidate:settings",
      "kind": "route_candidate",
      "path": "/settings",
      "callee": {
        "kind": "resolved",
        "symbol_id": "symbol:src/routes.tsx:jsonRoute"
      },
      "component": {
        "kind": "unresolved",
        "name": "Settings"
      },
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 20,
            "end_byte": 79,
            "start": {
              "line": 2,
              "column": 1
            },
            "end": {
              "line": 5,
              "column": 3
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:symbol:independent",
      "kind": "symbol",
      "symbol_id": "symbol:src/routes.tsx:Independent",
      "module_id": "module:src/routes.tsx",
      "name": "Independent",
      "namespace": "value",
      "symbol_kind": "component",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 80,
            "end_byte": 120,
            "start": {
              "line": 7,
              "column": 1
            },
            "end": {
              "line": 9,
              "column": 2
            }
          }
        }
      ]
    },
    {
      "finding_id": "finding:symbol:json-route",
      "kind": "symbol",
      "symbol_id": "symbol:src/routes.tsx:jsonRoute",
      "module_id": "module:src/routes.tsx",
      "name": "jsonRoute",
      "namespace": "value",
      "symbol_kind": "function",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 121,
            "end_byte": 130,
            "start": {
              "line": 11,
              "column": 10
            },
            "end": {
              "line": 11,
              "column": 19
            }
          }
        }
      ]
    }
  ],
  "diagnostics": [
    {
      "diagnostic_id": "diagnostic:unresolved-settings",
      "code": "unresolved_symbol",
      "category": "linkage",
      "effect": "incomplete",
      "severity": "error",
      "root_ids": [
        "root:src/routes.tsx"
      ],
      "subject": {
        "kind": "finding",
        "finding_id": "finding:route-candidate:settings"
      },
      "message": "the route component does not resolve to one symbol",
      "provenance": [
        {
          "source_id": "source:src/routes.tsx",
          "span": {
            "start_byte": 62,
            "end_byte": 70,
            "start": {
              "line": 4,
              "column": 14
            },
            "end": {
              "line": 4,
              "column": 22
            }
          }
        }
      ]
    }
  ]
}
```

## Representative failed result

The only root cannot be read, so no trustworthy findings exist.

```json
{
  "name": "query-analysis",
  "version": 1,
  "analysis_root": ".",
  "completeness": {
    "state": "failed",
    "reason_diagnostic_ids": [
      "diagnostic:missing-root"
    ]
  },
  "roots": [
    {
      "root_id": "root:src/missing.ts",
      "state": "failed",
      "reason_diagnostic_ids": [
        "diagnostic:missing-root"
      ]
    }
  ],
  "sources": [],
  "modules": [],
  "findings": [],
  "diagnostics": [
    {
      "diagnostic_id": "diagnostic:missing-root",
      "code": "source_unavailable",
      "category": "source",
      "effect": "fatal",
      "severity": "error",
      "root_ids": [
        "root:src/missing.ts"
      ],
      "subject": {
        "kind": "root",
        "root_id": "root:src/missing.ts"
      },
      "message": "the requested source is unavailable",
      "provenance": []
    }
  ]
}
```

## Required edge outcomes

- Two import findings can contain `./button` and `./button.tsx` while resolving
  to the same Module ID.
- An import alias resolves to the defining source-qualified Symbol ID.
- A re-export chain retains ordered export Finding IDs.
- An ambiguous star re-export retains sorted candidate Symbol IDs and produces
  a partial result.
- Module and component cycles produce cycle groups with supporting Finding IDs
  and provenance.
- A component self-loop remains a direct edge and a one-member cycle group.
- Duplicate route paths retain both route Finding IDs and produce a
  `duplicate_route` ambiguity.
- A static path with an unresolved component remains a route candidate and
  makes the result partial.
- A template path produces an `unsupported_syntax` diagnostic and no
  recognized route.
