import { Alert, Button, Flex, Progress, Typography } from "antd";
import { ArrowDownToLine } from "lucide-react";
import { useState, useSyncExternalStore } from "react";
import { AtlasModal } from "../shared/ui/overlays/AtlasModal";

type Phase =
  | { kind: "available" }
  | { kind: "downloading"; percent: number | null }
  | { kind: "installing" };

type Role = "primary" | "trial";
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
  | { kind: "install"; token: string };

type UpdateAttachment = {
  token: string;
  channel?: {
    installed: { tag: string; version: string };
    active: Role;
    trial_path: string | null;
    returning: { workspace: Role; updates: Waiting } | null;
    ready: boolean;
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
        okText={actionError ? "Try again" : "Install update"}
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

function workspaceName(role: Role) {
  return role === "primary" ? "primary workspace" : "trial workspace";
}

export function DesktopReleaseChannel() {
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
      <Button size="small" onClick={show}>
        {channel.ready ? "Stable available" : "Release channel"}
      </Button>
      <AtlasModal
        open={open}
        layout="confirmation"
        title="Atlas release channel"
        onCancel={() => setOpen(false)}
        footer={<Button onClick={() => setOpen(false)}>Close</Button>}
      >
        <Flex vertical gap="middle">
          <Typography.Text>
            Running {installedNightly ? "Nightly" : "Stable"}{" "}
            {channel.installed.version} with the {workspaceName(channel.active)}.
          </Typography.Text>
          {channel.trial_path && (
            <Typography.Text type="secondary">
              Terminal commands open the primary workspace by default. To use the trial,
              pass <code>--index</code> and the path <code>{channel.trial_path}</code>.
            </Typography.Text>
          )}
          {channel.returning && (
            <Alert
              type="info"
              title={
                channel.returning.updates === "hold"
                  ? `Nightly update offers are paused while Atlas waits for Stable for your ${workspaceName(channel.returning.workspace)}.`
                  : `Atlas is still offering Nightly updates and checking for Stable for your ${workspaceName(channel.returning.workspace)}.`
              }
            />
          )}
          <Typography.Text>
            Choose the workspace to open after switching to {target}.
          </Typography.Text>
          <Flex wrap gap="small">
            {(["primary", "trial"] as const).map((role) => (
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
                {role === "primary" ? "Primary workspace" : "Trial workspace"}
              </Button>
            ))}
          </Flex>
          {!installedNightly && workspace === "trial" && (
            <Typography.Text type="secondary">
              If you haven’t used the trial workspace yet, Atlas will start it with a
              copy of your primary workspace.
            </Typography.Text>
          )}
          {review ? (
            <>
              <Typography.Text>
                Atlas will install {target} {review.version} and open the{" "}
                {workspaceName(review.workspace)}.
              </Typography.Text>
              {review.will_seed && (
                <Alert
                  type="info"
                  title="Atlas will copy your primary workspace to start the trial."
                />
              )}
              {(review.needs_setup || review.will_upgrade) && (
                <Alert
                  type="info"
                  title="Atlas will ask you to prepare the selected workspace after restart."
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
          {(error ?? channel.error) && (
            <Alert type="error" title={error ?? channel.error} />
          )}
          {installedNightly && !review && (
            <Flex vertical align="start" gap="small">
              <Button
                disabled={disabled}
                onClick={() =>
                  void request({ kind: "wait", workspace, updates: "hold" })
                }
              >
                Stay on this build and wait for Stable
              </Button>
              <Button
                disabled={disabled}
                onClick={() =>
                  void request({ kind: "wait", workspace, updates: "continue" })
                }
              >
                Keep receiving Nightly updates while waiting
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
