# Optional MCP Compatibility Spike

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

After CLI plus skills are proven, is `atlas mcp` still worth maintaining as an optional compatibility surface?

## Motivation

MCP is less compelling if PF2e Atlas remains a local-only developer tool. It may still be useful as a compatibility mode, but it should not drive the runtime architecture unless it provides clear value.

## Prototype Scope

- Implement or sketch a thin Rust MCP surface over the same runtime services as the CLI.
- Cover only representative tools:
  - lookup
  - search
  - rule context
- Compare implementation complexity, schema maintenance, output size, and agent usefulness against CLI plus skills.
- Verify whether the Rust MCP SDK covers the needed stdio server behavior cleanly.

## Do Not Mock

- MCP tool schema definition
- request validation
- stdio transport
- runtime service reuse
- output shaping
- maintenance cost comparison

## Outputs

- recommendation to keep, defer, or drop MCP compatibility
- list of tools worth preserving if kept
- maintenance-cost notes
- explicit rule that MCP must remain a thin surface if retained

## Migration Dependency

This spike should come after the CLI agent surface spike. MCP should not block the runtime migration unless there is a real external-client requirement.
