import { Alert, Button, Flex, Progress, Typography } from "antd";
import { ArrowDownToLine } from "lucide-react";
import { useState, useSyncExternalStore } from "react";
import { AtlasModal } from "../shared/ui/overlays/AtlasModal";
import "./desktopReleaseChannel.css";

type Phase =
  | { kind: "available" }
  | { kind: "downloading"; percent: number | null }
  | { kind: "installing" };

type Role = "original" | "separate";
type Waiting = "hold" | "continue";
type ChannelReview = {
  id: string;
  version: string;
  workspace: Role;
  will_seed: boolean;
  needs_setup: boolean;
  will_upgrade: boolean;
};
type ChannelAction =
  | { kind: "review"; workspace: Role; seed_if_missing: boolean }
  | { kind: "wait"; workspace: Role; updates: Waiting }
  | { kind: "cancel_wait" }
  | { kind: "retry_pending" }
  | { kind: "install"; token: string };

type UpdateAttachment = {
  token: string;
  channel?: {
    installed: { tag: string; version: string };
    active: Role;
    workspace_path: string | null;
    other_exists: boolean;
    returning: { workspace: Role; updates: Waiting } | null;
    ready: boolean;
    pending_version?: string | null;
    busy: boolean;
    error?: string | null;
  } | null;
  update?: { version: string; phase: Phase; error: string | null } | null;
};

type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: UpdateAttachment;
  __TAURI__?: {
    core: {
      invoke: {
        (
          command: "desktop_update_action",
          args: { action: "details" | "install"; token: string; version: string },
        ): Promise<string | null>;
        (
          command: "desktop_channel_action",
          args: { action: ChannelAction; token: string },
        ): Promise<ChannelReview | null>;
      };
    };
  };
};

function readAttachment() {
  return (window as DesktopWindow).__ATLAS_DESKTOP_SETUP__;
}

function subscribe(callback: () => void) {
  window.addEventListener("atlas-desktop-setup", callback);
  return () => window.removeEventListener("atlas-desktop-setup", callback);
}

const fallbackNotice =
  "If this update requires setup, Atlas may be temporarily unavailable after restarting.";

export function DesktopUpdate() {
  const attachment = useSyncExternalStore(subscribe, readAttachment);
  const [selected, select] = useState<{
    version: string;
    notice?: string | null;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const update = attachment?.update;
  const host = (window as DesktopWindow).__TAURI__?.core;

  if (!attachment || !update || !host) return null;
  const { token } = attachment;
  const { version, phase } = update;
  const current = selected?.version === version;
  const loadingDetails = selected?.notice === undefined;
  const active = phase.kind !== "available";

  async function open() {
    const request = { version };
    select(request);
    setError(null);
    const notice = await host!
      .invoke("desktop_update_action", {
        action: "details",
        token,
        version,
      })
      .catch(() => null);
    select((current) => (current === request ? { ...request, notice } : current));
  }

  async function install() {
    if (sending || active || !current) return;
    setSending(true);
    setError(null);
    try {
      await host!.invoke("desktop_update_action", {
        action: "install",
        token,
        version,
      });
    } catch (failure) {
      setError(
        typeof failure === "string" ? failure : "Atlas couldn’t start the update.",
      );
    } finally {
      setSending(false);
    }
  }

  const actionError = error ?? update.error;
  return (
    <>
      <Button
        type="primary"
        size="small"
        icon={<ArrowDownToLine size={15} />}
        onClick={open}
      >
        Update
      </Button>
      <AtlasModal
        open={selected !== null}
        layout="confirmation"
        title="Update Atlas Desktop"
        onCancel={() => select(null)}
        cancelText={active ? "Close" : "Later"}
        okText="Install update"
        okButtonProps={{
          disabled: !current || active || loadingDetails,
          loading: sending,
        }}
        onOk={install}
      >
        <Flex gap="middle" vertical>
          {current ? (
            <>
              <Typography.Text>
                Atlas {version} is available. Atlas will restart to finish installing
                it.
              </Typography.Text>
              {!loadingDetails && (
                <Alert type="info" title={selected?.notice ?? fallbackNotice} />
              )}
              {phase.kind === "downloading" && (
                <div role="status">
                  Downloading update…
                  {phase.percent !== null && <Progress percent={phase.percent} />}
                </div>
              )}
              {phase.kind === "installing" && (
                <div role="status">Installing and restarting…</div>
              )}
              {actionError && <Alert type="error" title={actionError} />}
            </>
          ) : (
            <Alert
              type="info"
              title="A different version is available. Close this dialog and open Update again."
            />
          )}
        </Flex>
      </AtlasModal>
    </>
  );
}

export function DesktopStableUpdate() {
  return <DesktopReleaseChannel offerOnly />;
}

export function DesktopReleaseChannel({ offerOnly = false }: { offerOnly?: boolean }) {
  const attachment = useSyncExternalStore(subscribe, readAttachment);
  const host = (window as DesktopWindow).__TAURI__?.core;
  const [open, setOpen] = useState(false);
  const [selection, setSelection] = useState<Role | null>(null);
  const [review, setReview] = useState<ChannelReview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const channel = attachment?.channel;
  if (!attachment || !channel || !host) return null;
  const installedNightly = channel.installed.tag.startsWith("nightly/");
  const target = installedNightly ? "Stable" : "Nightly";
  const workspace = selection ?? channel.returning?.workspace ?? channel.active;
  const disabled = busy || channel.busy;
  const other: Role = channel.active === "original" ? "separate" : "original";
  const channelName = installedNightly ? "Nightly" : "Stable";
  if (offerOnly && !channel.ready && !open) return null;

  async function request(action: ChannelAction) {
    setBusy(true);
    setError(null);
    try {
      const result = await host!.invoke("desktop_channel_action", {
        token: attachment!.token,
        action,
      });
      setReview(result);
    } catch (failure) {
      setError(
        typeof failure === "string"
          ? failure
          : "Atlas couldn’t complete the channel request.",
      );
    } finally {
      setBusy(false);
    }
  }

  function show() {
    setSelection(null);
    setReview(null);
    setError(null);
    setOpen(true);
  }

  return (
    <>
      {offerOnly ? (
        <Button size="small" onClick={show}>
          Stable available
        </Button>
      ) : (
        <section
          className="desktop-setup__section"
          aria-labelledby="desktop-app-updates"
        >
          <h2 id="desktop-app-updates">App updates</h2>
          <Flex vertical gap="small" align="start">
            <Typography.Text strong>{channelName}</Typography.Text>
            <Typography.Text type="secondary">
              Version {channel.installed.version}
            </Typography.Text>
            {installedNightly || channel.pending_version || channel.ready ? (
              <Button onClick={show}>
                {channel.ready ? "Stable available" : "Change release channel"}
              </Button>
            ) : (
              <details className="desktop-channel__preview">
                <summary>Preview builds</summary>
                <Button onClick={show}>Try Nightly…</Button>
              </details>
            )}
          </Flex>
        </section>
      )}
      <AtlasModal
        open={open}
        layout="form"
        title="Change release channel"
        onCancel={() => setOpen(false)}
        footer={null}
      >
        <Flex vertical gap="middle">
          <Typography.Text strong>{channelName}</Typography.Text>
          <Typography.Text type="secondary">
            Version {channel.installed.version}
          </Typography.Text>
          {channel.returning && (
            <Alert
              type="info"
              title={
                channel.returning.updates === "hold"
                  ? "Waiting for a compatible Stable release. Not checking for Nightly updates."
                  : "Waiting for a compatible Stable release. Still checking for Nightly updates."
              }
            />
          )}
          {channel.pending_version ? (
            <>
              <Alert
                type="info"
                title={`Atlas ${channel.pending_version} still has a pending installation.`}
              />
              <Button
                type="primary"
                disabled={disabled}
                onClick={() => void request({ kind: "retry_pending" })}
              >
                Retry pending install
              </Button>
            </>
          ) : (
            <>
              <Typography.Text>
                Choose which data to use after switching to {target}.
              </Typography.Text>
              <Flex wrap gap="small" className="desktop-channel__choices">
                {[
                  channel.active,
                  ...(!installedNightly || channel.other_exists ? [other] : []),
                ].map((role) => (
                  <Button
                    key={role}
                    aria-pressed={workspace === role}
                    color={workspace === role ? "primary" : "default"}
                    variant={workspace === role ? "filled" : "outlined"}
                    disabled={disabled}
                    onClick={() => {
                      setSelection(role);
                      setReview(null);
                      setError(null);
                    }}
                  >
                    {role === channel.active
                      ? "Use my current data"
                      : installedNightly
                        ? "Use an earlier copy"
                        : channel.other_exists
                          ? "Use the other copy"
                          : "Use a separate copy"}
                  </Button>
                ))}
              </Flex>
              {installedNightly && (
                <Typography.Text type="secondary">
                  {workspace === channel.active
                    ? "Keep the data you’ve been using in Nightly, including your latest changes."
                    : "Use an earlier, separate copy of your data. Changes made in Nightly won’t be copied over. Your current Nightly data will remain available."}
                </Typography.Text>
              )}
              {!installedNightly && workspace !== channel.active && (
                <Typography.Text type="secondary">
                  {(review ? !review.will_seed : channel.other_exists)
                    ? "Reopens your other copy with its earlier changes. Neither copy is overwritten."
                    : "Creates a copy of your current data. Changes made to either copy stay separate."}
                </Typography.Text>
              )}
              {review ? (
                <>
                  <Typography.Text>
                    Atlas will install {target} {review.version} and use{" "}
                    {review.workspace === channel.active
                      ? "your current data"
                      : review.will_seed
                        ? "your separate copy"
                        : "an earlier copy of your data"}
                    .
                  </Typography.Text>
                  <Typography.Text type="secondary">
                    After switching, the app and its bundled CLI will use your existing
                    data.
                  </Typography.Text>
                  {(review.needs_setup || review.will_upgrade) && (
                    <Alert
                      type="info"
                      title="Atlas will ask you to prepare your data after restarting."
                    />
                  )}
                  <Button
                    type="primary"
                    disabled={disabled}
                    onClick={() => void request({ kind: "install", token: review.id })}
                  >
                    {disabled ? "Installing…" : `Install ${target} and restart`}
                  </Button>
                </>
              ) : (
                <Button
                  type="primary"
                  loading={busy}
                  disabled={channel.busy}
                  onClick={() =>
                    void request({
                      kind: "review",
                      workspace,
                      seed_if_missing: !installedNightly,
                    })
                  }
                >
                  {channel.busy ? "Installing…" : `Check ${target} compatibility`}
                </Button>
              )}
            </>
          )}
          {(error ?? channel.error) && (
            <Alert type="error" title={error ?? channel.error} />
          )}
          {installedNightly && !review && !channel.pending_version && (
            <Flex
              vertical
              align="start"
              gap="small"
              className="desktop-channel__waiting"
            >
              <Button
                disabled={disabled}
                onClick={() =>
                  void request({ kind: "wait", workspace, updates: "hold" })
                }
              >
                Stay on this build and wait for the next Stable release
              </Button>
              <Button
                disabled={disabled}
                onClick={() =>
                  void request({ kind: "wait", workspace, updates: "continue" })
                }
              >
                Keep checking for Nightly updates while waiting
              </Button>
              {channel.returning && (
                <Button
                  type="text"
                  disabled={disabled}
                  onClick={() => void request({ kind: "cancel_wait" })}
                >
                  Cancel return to Stable
                </Button>
              )}
            </Flex>
          )}
        </Flex>
      </AtlasModal>
    </>
  );
}
