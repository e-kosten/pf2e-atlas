# Vendor Data

This directory holds large local dependencies that are required at runtime but are intentionally not tracked in this repository.

## PF2E Data Checkout

The MCP server expects the Foundry PF2E system checkout at:

```text
vendor/pf2e
```

Populate it with:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

The `vendor/pf2e` checkout is ignored by this repo's Git history on purpose:

- it is large
- it changes independently of this MCP server
- it is treated as local runtime data, not project source

The MCP server will:

- read data from `vendor/pf2e` by default
- attempt a best-effort `git pull --ff-only` on startup
- continue using the existing local checkout if refresh fails
