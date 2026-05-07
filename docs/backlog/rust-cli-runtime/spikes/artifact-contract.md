# Artifact Contract Spike

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

What versioned artifact contract should separate offline prep from the Rust runtime?

## Motivation

Python or Node prep is low-cost only if runtime artifacts are durable, validated, and explicit. Without a strong artifact contract, cross-language prep can drift away from runtime assumptions.

## Prototype Scope

- Define a minimal versioned SQLite schema contract for runtime consumption.
- Define embedding identity metadata required to validate query/document compatibility.
- Define JSON/JSONL schemas for reports or review artifacts that should not live directly in SQLite.
- Build tiny golden fixtures and validation commands.
- Include source signature, schema version, embedding identity, and search artifact metadata.

## Do Not Mock

- schema version checks
- source signature checks
- embedding model identity
- tokenizer/pooling/normalization metadata
- CLI output fixtures
- stale-artifact diagnostics

## Outputs

- artifact contract draft
- fixture corpus
- validation command sketch
- recommendation for what belongs in SQLite versus adjacent JSON/JSONL

## Migration Dependency

No runtime migration implementation should begin until this spike defines the boundary that Rust will validate at startup.
