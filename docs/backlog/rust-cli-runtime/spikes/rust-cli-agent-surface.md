# Rust CLI Agent Surface Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Can a Rust `atlas` CLI plus Codex skills serve local agent workflows more reliably and compactly than the current MCP-first surface?

## Motivation

The product is local-only in practice. Agents may do better with stable commands, compact JSON output, predictable exit codes, and procedural skill guidance than with broad MCP tool schemas.

## Prototype Scope

- Implement a disposable Rust CLI with representative commands:
  - `atlas lookup`
  - `atlas search`
  - `atlas rule-context`
- Use fixture data or the existing prepared index.
- Return stable compact JSON by default.
- Add concise stderr diagnostics and meaningful exit codes.
- Draft a minimal Codex skill that teaches when to use each command.
- Run a few real agent tasks through the CLI and compare friction against MCP.

## Do Not Mock

- command names and option shape
- JSON output structure
- error behavior
- exit codes
- agent invocation from a skill
- response size

## Outputs

- CLI command contract recommendation
- JSON output examples
- skill draft notes
- observed agent ergonomics
- keep, kill, or adjust recommendation

## Migration Dependency

If the CLI surface is not clearly better for local agents, the migration should reconsider whether MCP remains the primary runtime surface.
