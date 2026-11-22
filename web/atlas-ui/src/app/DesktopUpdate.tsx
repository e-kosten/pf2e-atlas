import { Alert, Button, Flex, Progress, Spin, Typography } from "antd";
import { ArrowDownToLine } from "lucide-react";
import { type RefObject, useEffect, useLayoutEffect, useRef, useState } from "react";
import { invokeDesktop, useDesktopAttachment } from "./desktopBridge";
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

export type UpdateAttachment = {
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
  update?: {
    version: string;
    phase: Phase;
    error: string | null;
    switching?: boolean;
  } | null;
};

type Offer = NonNullable<UpdateAttachment["update"]>;
type Channel = NonNullable<UpdateAttachment["channel"]>;
type Impact =
  { kind: "restart" | "upgrade" | "unknown" } | { kind: "reference"; version: string };

function progressLabel(phase: Exclude<Phase, { kind: "available" }>) {
  if (phase.kind === "installing") return "Installing and restarting…";
  return phase.percent === null
    ? "Downloading update…"
    : `Downloading update · ${phase.percent}%`;
}

export function DesktopUpdateShortcut({
  onOpen,
}: {
  onOpen: (destination: "updates" | "channel") => void;
}) {
  const attachment = useDesktopAttachment();
  const update = attachment?.update;
  const channel = attachment?.channel;
  const active = update && update.phase.kind !== "available" ? update.phase : null;
  const showChannel =
    !active && (channel?.pending_version || channel?.ready || channel?.error);
  const label = active
    ? progressLabel(active)
    : showChannel
      ? channel?.pending_version
        ? "Install pending"
        : channel?.error
          ? "Review update"
          : "Stable available"
      : update?.error
        ? "Retry update"
        : update
          ? "Update available"
          : null;
  if (!label) return null;
  return (
    <Button
      size="small"
      type="primary"
      icon={<ArrowDownToLine size={15} />}
      onClick={() => onOpen(showChannel ? "channel" : "updates")}
    >
      {label}
    </Button>
  );
}

type InstallFocus = RefObject<{ token: string; version: string } | null>;

function rememberInstallFocus(
  focusRef: InstallFocus,
  button: HTMLElement,
  token: string,
  version: string,
) {
  focusRef.current = button === document.activeElement ? { token, version } : null;
}

function UpdateProgress({
  phase,
  version,
  token,
  installFocusRef,
}: {
  phase: Exclude<Phase, { kind: "available" }>;
  version: string;
  token: string;
  installFocusRef: InstallFocus;
}) {
  const status = useRef<HTMLSpanElement>(null);
  useLayoutEffect(() => {
    const intended = installFocusRef.current;
    installFocusRef.current = null;
    if (
      intended?.version === version &&
      intended.token === token &&
      document.hasFocus() &&
      document.activeElement === document.body &&
      !status.current?.closest("[hidden], [inert]")
    )
      status.current?.focus();
  }, [installFocusRef, token, version]);
  return (
    <Flex vertical gap="small">
      <Flex align="center" gap="small">
        {(phase.kind === "installing" || phase.percent === null) && (
          <Spin size="small" />
        )}
        <span
          ref={status}
          role="status"
          tabIndex={-1}
          className="desktop-update__phase"
        >
          {phase.kind === "downloading"
            ? "Downloading update…"
            : "Installing and restarting…"}
        </span>
        {phase.kind === "downloading" && phase.percent !== null && (
          <Typography.Text aria-hidden="true">{phase.percent}%</Typography.Text>
        )}
      </Flex>
      <Typography.Text type="secondary">Atlas {version}</Typography.Text>
      {phase.kind === "downloading" && phase.percent !== null && (
        <Progress
          status="normal"
          aria-label="Update download"
          percent={phase.percent}
          showInfo={false}
        />
      )}
    </Flex>
  );
}

function impactNotice(impact: Impact) {
  switch (impact.kind) {
    case "restart":
      return null;
    case "upgrade":
      return "Atlas will upgrade your workspace after restarting. You can reopen it when the upgrade finishes.";
    case "reference":
      return `This build includes Foundry PF2e ${impact.version}. You can apply its reference data after restarting.`;
    case "unknown":
      return "Atlas couldn’t determine whether your workspace will need setup after restarting.";
  }
}

function UpdateOffer({
  token,
  update,
  installed,
  setupRunning,
  installFocusRef,
}: {
  token: string;
  update: Offer;
  installed?: string;
  setupRunning: boolean;
  installFocusRef: InstallFocus;
}) {
  const [impact, setImpact] = useState<Impact>();
  const [error, setError] = useState<string>();
  const [check, setCheck] = useState(0);
  const [sending, setSending] = useState(false);
  const { version } = update;
  useEffect(() => {
    let current = true;
    void invokeDesktop<Impact>("desktop_update_action", {
      action: "details",
      token,
      version,
    })
      .then((result) => {
        if (current) setImpact(result);
      })
      .catch((failure) => {
        if (current)
          setError(failureMessage(failure, "Atlas couldn’t check this update."));
      });
    return () => {
      current = false;
    };
  }, [check, token, version]);
  async function install(button: HTMLElement) {
    if (sending || !impact || setupRunning) return;
    rememberInstallFocus(installFocusRef, button, token, version);
    setSending(true);
    setError(undefined);
    try {
      await invokeDesktop("desktop_update_action", {
        action: "install",
        token,
        version,
      });
    } catch (failure) {
      installFocusRef.current = null;
      setError(failureMessage(failure, "Atlas couldn’t start the update."));
    } finally {
      setSending(false);
    }
  }
  const message = impact && impactNotice(impact);
  const failure = error ?? update.error;
  return (
    <Flex vertical gap="small">
      <div className="desktop-update__row">
        <div>
          <Typography.Text strong>Atlas {version} is available</Typography.Text>
          {installed && (
            <div>
              <Typography.Text type="secondary">
                Installed version {installed}
              </Typography.Text>
            </div>
          )}
        </div>
        <Button
          type="primary"
          loading={sending}
          disabled={!impact || setupRunning}
          onClick={(event) => void install(event.currentTarget)}
        >
          {update.error ? "Retry install and restart" : "Install and restart"}
        </Button>
      </div>
      {!impact && !error && (
        <Typography.Text type="secondary" role="status">
          Checking update…
        </Typography.Text>
      )}
      {setupRunning && (
        <Typography.Text type="secondary">
          Finish setup before installing the update.
        </Typography.Text>
      )}
      {message && <Typography.Text>{message}</Typography.Text>}
      {failure && (
        <Alert
          type="error"
          title={failure}
          action={
            !impact && (
              <Button
                onClick={() => {
                  setError(undefined);
                  setCheck((value) => value + 1);
                }}
              >
                Retry check
              </Button>
            )
          }
        />
      )}
    </Flex>
  );
}

export function DesktopUpdateSection() {
  const attachment = useDesktopAttachment();
  const installFocusRef: InstallFocus = useRef(null);
  if (!attachment) return null;
  const { update, channel, token, setup } = attachment;
  if (update && update.phase.kind !== "available")
    return (
      <UpdateProgress
        phase={update.phase}
        version={update.version}
        token={token}
        installFocusRef={installFocusRef}
      />
    );
  if (update)
    return (
      <UpdateOffer
        key={JSON.stringify([token, update.version])}
        update={update}
        installFocusRef={installFocusRef}
        token={token}
        installed={channel?.installed.version}
        setupRunning={setup?.status === "running" || setup?.status === "cancelling"}
      />
    );
  return (
    <Typography.Text>
      {channel
        ? `Installed version ${channel.installed.version}`
        : "Update information is unavailable."}
    </Typography.Text>
  );
}

function failureMessage(failure: unknown, fallback: string) {
  if (typeof failure === "string") return failure;
  if (
    failure &&
    typeof failure === "object" &&
    "message" in failure &&
    typeof failure.message === "string"
  )
    return failure.message;
  return fallback;
}

export function DesktopReleaseChannel() {
  const attachment = useDesktopAttachment();
  const installFocusRef: InstallFocus = useRef(null);
  if (attachment?.update && attachment.update.phase.kind !== "available")
    return (
      <UpdateProgress
        phase={attachment.update.phase}
        version={attachment.update.version}
        token={attachment.token}
        installFocusRef={installFocusRef}
      />
    );
  if (!attachment) return null;
  if (!attachment.channel)
    return (
      <Typography.Text>Release channel information is unavailable.</Typography.Text>
    );
  const { token, channel } = attachment;
  return (
    <ChannelOptions
      key={JSON.stringify([
        token,
        channel.installed.tag,
        channel.active,
        channel.other_exists,
        channel.returning,
        channel.pending_version,
      ])}
      channel={channel}
      token={token}
      installFocusRef={installFocusRef}
    />
  );
}

function ChannelOptions({
  channel,
  token,
  installFocusRef,
}: {
  channel: Channel;
  token: string;
  installFocusRef: InstallFocus;
}) {
  const [selection, setSelection] = useState<Role | null>(null);
  const [review, setReview] = useState<ChannelReview | null>(null);
  const [busy, setBusy] = useState(false);
  const [unavailable, setUnavailable] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const installedNightly = channel.installed.tag.startsWith("nightly/");
  const target = installedNightly ? "Stable" : "Nightly";
  const choosingWorkspace = selection !== null && !channel.returning;
  const currentDataButton = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    if (choosingWorkspace) currentDataButton.current?.focus();
  }, [choosingWorkspace]);
  const workspace = selection ?? channel.returning?.workspace ?? channel.active;
  const disabled = busy || channel.busy;
  const missingSelection =
    installedNightly && workspace !== channel.active && !channel.other_exists;
  const other: Role = channel.active === "original" ? "separate" : "original";
  async function request(action: ChannelAction, button?: HTMLElement) {
    if (disabled) return;
    const targetVersion =
      action.kind === "retry_pending" ? channel.pending_version : review?.version;
    if (button && targetVersion)
      rememberInstallFocus(installFocusRef, button, token, targetVersion);
    setBusy(true);
    setError(null);
    setUnavailable(false);
    try {
      setReview(
        await invokeDesktop<ChannelReview | null>("desktop_channel_action", {
          token,
          action,
        }),
      );
    } catch (failure) {
      if (button) installFocusRef.current = null;
      setReview(null);
      setError(failureMessage(failure, "Atlas couldn’t complete the channel request."));
      setUnavailable(
        action.kind === "review" &&
          failure !== null &&
          typeof failure === "object" &&
          "kind" in failure &&
          failure.kind === "unavailable",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <Flex vertical gap="middle" className="desktop-channel">
      <Typography.Text type="secondary">
        Current channel · {installedNightly ? "Nightly" : "Stable"}
      </Typography.Text>
      {channel.returning && (
        <Alert
          type="info"
          title={
            channel.ready
              ? "A compatible Stable release is ready to review."
              : channel.returning.updates === "hold"
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
            onClick={(event) =>
              void request({ kind: "retry_pending" }, event.currentTarget)
            }
          >
            Retry pending install
          </Button>
        </>
      ) : selection === null && !channel.returning ? (
        <Button disabled={disabled} onClick={() => setSelection(channel.active)}>
          Switch to {target}…
        </Button>
      ) : (
        <>
          <Typography.Text>
            Choose which data to use after switching to {target}.
          </Typography.Text>
          {missingSelection && (
            <Alert
              type="error"
              title="The earlier copy of your data is missing. Choose your current data to continue."
            />
          )}
          <Flex wrap gap="small" className="desktop-channel__choices">
            {[
              channel.active,
              ...(!installedNightly || channel.other_exists ? [other] : []),
            ].map((role) => (
              <Button
                key={role}
                ref={role === channel.active ? currentDataButton : undefined}
                aria-pressed={workspace === role}
                color={workspace === role ? "primary" : "default"}
                variant={workspace === role ? "filled" : "outlined"}
                disabled={disabled}
                onClick={() => {
                  setSelection(role);
                  setUnavailable(false);
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
          {installedNightly && !missingSelection && (
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
              {(review.needs_setup || review.will_upgrade) && (
                <Alert
                  type="info"
                  title={
                    review.will_upgrade
                      ? "This version needs to upgrade your data after restarting. Atlas will be unavailable until the upgrade finishes."
                      : "Atlas needs to set up your data after restarting. The app will be unavailable until setup finishes."
                  }
                />
              )}
              <Button
                type="primary"
                disabled={disabled || missingSelection}
                onClick={(event) =>
                  void request(
                    { kind: "install", token: review.id },
                    event.currentTarget,
                  )
                }
              >
                {disabled ? "Installing…" : `Install ${target} and restart`}
              </Button>
            </>
          ) : (
            <Button
              type="primary"
              loading={busy}
              disabled={channel.busy || missingSelection}
              onClick={() =>
                void request({
                  kind: "review",
                  workspace,
                  seed_if_missing: !installedNightly,
                })
              }
            >
              {`Check ${target} compatibility`}
            </Button>
          )}
        </>
      )}
      {(error ?? channel.error) && (
        <Alert type="error" title={error ?? channel.error} />
      )}
      {installedNightly &&
        !review &&
        !channel.pending_version &&
        (unavailable || channel.returning) && (
          <Flex vertical align="start" gap="small" className="desktop-channel__waiting">
            <Button
              disabled={disabled || missingSelection}
              onClick={() => void request({ kind: "wait", workspace, updates: "hold" })}
            >
              Stay on this build and wait for the next Stable release
            </Button>
            <Button
              disabled={disabled || missingSelection}
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
  );
}
