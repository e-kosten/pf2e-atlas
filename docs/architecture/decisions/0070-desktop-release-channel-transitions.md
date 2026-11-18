# ADR0070 Desktop release channel transitions

## Decision

Atlas Desktop follows the channel of the installed build. A user may explicitly
review and install a different channel. A pending return to Stable records the
intended workspace and whether Nightly update offers should continue. Desktop
preferences and pending switch recovery live in one versioned, atomically
updated configuration file outside SQLite.

The existing workspace is the initial default. A separate managed workspace is
an opt-in snapshot with a new identity. Runtime owns the two fixed managed paths
and one versioned, atomically replaced selection/journal in application-data
storage beside the workspaces. The lightweight desktop protocol owns its closed
messages; the host reads and saves them through the bundled CLI with revision
comparison and a short configuration lock. There is no mirrored desktop-only
selection.

Desktop and current-version global CLI commands follow the same committed
selection. Explicit paths and repository mode take precedence. The host pins
its selected path for all subprocesses, including setup, so an already running
app cannot silently move to a different workspace. A switch commits the shared
selection only after the expected target build has opened the reviewed workspace
service successfully. Pending installation, setup refusal or resuming the previous
build leaves the prior default intact. Existing CLI processes retain their paths.
A missing selected workspace or invalid configuration fails closed; explicit
paths and protected native recovery remain available.

The copy is durable and can remain selected on Stable. The original workspace is
preserved. Repeated switches can reopen the other retained workspace, with clear
reuse wording rather than promising to make a fresh copy. The selection is
independent of the release channel. Older standalone binaries cannot honor this
new setting and require an explicit path or upgrade.

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

A compatible channel switch can continue the same active campaign. A separate
workspace remains available across installs. Workspaces may diverge through independent
edits. The recovery archive remains available for explicit saved-data transfer
into an empty workspace. Installing a different binary manually uses that
build's startup checks and configuration capabilities.
