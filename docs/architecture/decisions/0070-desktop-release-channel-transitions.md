# ADR0070 Desktop release channel transitions

## Decision

Atlas Desktop follows the channel of the installed build. A user may explicitly
review and install a different channel. A pending return to Stable records the
intended workspace and whether Nightly update offers should continue. Desktop
preferences and pending switch recovery live in one versioned, atomically
updated configuration file outside SQLite.

The existing primary workspace remains the default. A separate managed trial
is an opt-in snapshot with a new workspace identity. The desktop host passes the
selected workspace path to its bundled CLI. Standalone and linked terminal
commands retain the primary default and accept the trial path explicitly.

The existing publisher key authenticates both the updater package and a
detached descriptor that binds its digest and release identity to the
packaged CLI's migration ledger. Storage authenticates workspace history using
the same comparison it uses at ordinary startup. A matching older prefix
permits a supported upgrade subject to the existing migration validation,
backups and recovery. Historical releases without proof remain eligible for
ordinary primary-workspace updates.

The host records a reviewed destination before installing. The incoming build
checks the expected release and workspace identity under storage admission
before routing probes and setup, and commits selection after the workspace
service starts successfully. On Windows the bundled CLI exits before NSIS
replaces it. Startup repeats admission to handle a concurrent change during
that handoff. Protected bootstrap recovery is available before a workspace
service starts.

## Consequences

A compatible channel switch can continue the same active campaign. A trial
remains available across installs. Workspaces may diverge through independent
edits. The recovery archive remains available for explicit saved-data transfer
into an empty workspace. Installing a different binary manually uses that
build's startup checks and configuration capabilities.
