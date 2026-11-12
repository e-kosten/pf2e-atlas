import { Alert, Button, Progress, Space, Typography } from "antd";
import { useEffect, useState, useSyncExternalStore } from "react";
import { AtlasDrawer } from "../shared/ui/overlays/AtlasDrawer";

type SetupStatus = "idle" | "available" | "running" | "cancelling" | "failed";
type SetupAction =
  | "cancel"
  | "resume"
  | "build_locally"
  | "apply_reference"
  | "check_reference"
  | "read_reference_status";
type DesktopAttachment = {
  token: string;
  openRequest?: number | null;
  referenceLoading?: boolean;
  reference?: {
    revision: string | null;
    target: string;
    targetVersion: string;
    managed: boolean;
    available: boolean;
  } | null;
  setup: {
    job: number;
    sequence: number;
    status: SetupStatus;
    progress?: { message?: string; current?: number; total?: number; unit?: string };
    message?: string;
    local_build: boolean;
    can_resume: boolean;
    reference_job?: boolean;
  };
};
type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: DesktopAttachment;
  __TAURI__?: {
    core: { invoke: (command: string, args: object) => Promise<void> };
  };
};
function readAttachment() {
  return (window as DesktopWindow).__ATLAS_DESKTOP_SETUP__;
}
function subscribe(callback: () => void) {
  window.addEventListener("atlas-desktop-setup", callback);
  return () => window.removeEventListener("atlas-desktop-setup", callback);
}

export function DesktopSetupPanel() {
  const attachment = useSyncExternalStore(subscribe, readAttachment);
  const [open, setOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [lastOpenRequest, setLastOpenRequest] = useState<number>();
  const [confirmation, setConfirmation] = useState<number>();
  const setup = attachment?.setup;
  const token = attachment?.token;
  const job = setup?.job;
  const openRequest = attachment?.openRequest;
  if (openRequest != null && openRequest !== lastOpenRequest) {
    setLastOpenRequest(openRequest);
    setConfirmation(undefined);
    setOpen(true);
  }
  useEffect(() => {
    if (openRequest == null || !token || job == null) return;
    void (window as DesktopWindow).__TAURI__?.core
      .invoke("desktop_setup_action", {
        action: "acknowledge_open",
        token,
        job,
        openRequest,
      })
      .catch(() => {});
  }, [openRequest, token, job]);
  if (!attachment || !setup) return null;
  const { status, progress } = setup;
  const { reference, referenceLoading } = attachment;
  const running = status === "running" || status === "cancelling";
  const pendingPin = reference && reference.revision !== reference.target;
  const confirming = confirmation === setup.sequence;
  const percent =
    typeof progress?.current === "number" &&
    typeof progress.total === "number" &&
    progress.total > 0
      ? Math.min(100, Math.floor((100 * progress.current) / progress.total))
      : undefined;

  async function act(action: SetupAction) {
    const host = (window as DesktopWindow).__TAURI__;
    if (!host || sending || !attachment) return;
    setError(null);
    setSending(true);
    try {
      await host.core.invoke("desktop_setup_action", {
        action,
        token: attachment.token,
        job: attachment.setup.job,
        sequence: attachment.setup.sequence,
      });
    } catch (failure) {
      setError(
        typeof failure === "string"
          ? failure
          : "Atlas could not perform this setup action.",
      );
    } finally {
      setSending(false);
    }
  }

  return (
    <>
      <Button
        type="text"
        onClick={() => {
          setConfirmation(undefined);
          setOpen(true);
          if (!running) void act("read_reference_status");
        }}
      >
        Setup and Updates
      </Button>
      <AtlasDrawer title="Setup and Updates" open={open} onClose={() => setOpen(false)}>
        <Space vertical size="middle">
          {reference && (
            <Space vertical size="small">
              <Typography.Text strong>Reference data</Typography.Text>
              <Typography.Text>
                Current:{" "}
                {pendingPin
                  ? (reference.revision?.slice(0, 12) ?? "Unknown revision")
                  : `Foundry PF2e ${reference.targetVersion}`}
              </Typography.Text>
              {pendingPin && (
                <Typography.Text>
                  Bundled target: Foundry PF2e {reference.targetVersion}
                </Typography.Text>
              )}
              {!reference.managed && (
                <Typography.Text>
                  Use the Atlas CLI to maintain custom reference sources.
                </Typography.Text>
              )}
            </Space>
          )}
          {!running && referenceLoading && (
            <Typography.Text role="status">Checking reference status…</Typography.Text>
          )}
          {!running && !referenceLoading && !reference && !setup.can_resume && (
            <Button loading={sending} onClick={() => void act("read_reference_status")}>
              Check reference status
            </Button>
          )}
          {running && (
            <>
              <Typography.Text role="status">
                {status === "cancelling"
                  ? "Cancelling setup…"
                  : (progress?.message ??
                    (setup.reference_job
                      ? "Preparing reference data…"
                      : "Preparing semantic search…"))}
              </Typography.Text>
              {percent !== undefined && <Progress percent={percent} />}
              {typeof progress?.current === "number" && progress.unit && (
                <Typography.Text>
                  {percent === undefined
                    ? `${progress.current.toLocaleString()} ${progress.unit} completed`
                    : `${progress.current.toLocaleString()} of ${progress.total?.toLocaleString()} ${progress.unit}`}
                </Typography.Text>
              )}
              <Button
                disabled={sending || status === "cancelling"}
                onClick={() => void act("cancel")}
              >
                Cancel setup
              </Button>
            </>
          )}
          {status === "idle" && setup.message && (
            <Typography.Text role="status">{setup.message}</Typography.Text>
          )}
          {setup.message && status === "failed" && (
            <Alert type="warning" title={setup.message} />
          )}
          {error && <Alert type="error" title={error} />}
          {setup.can_resume && (status === "available" || status === "failed") && (
            <Button type="primary" loading={sending} onClick={() => void act("resume")}>
              {status === "failed" ? "Retry setup" : "Resume semantic setup"}
            </Button>
          )}
          {setup.local_build &&
            status === "failed" &&
            (setup.can_resume || reference?.available) &&
            !referenceLoading && (
              <Button disabled={sending} onClick={() => void act("build_locally")}>
                Build embeddings locally
              </Button>
            )}
          {!running &&
            !setup.can_resume &&
            reference?.available &&
            !referenceLoading &&
            (confirming ? (
              <Alert
                type="warning"
                title={`Apply Foundry PF2e ${reference.targetVersion}?`}
                description={`This replaces reference revision ${reference.revision?.slice(0, 12)} with the bundled version, which may be older.`}
                action={
                  <Space vertical>
                    <Button
                      type="primary"
                      loading={sending}
                      onClick={() => {
                        setConfirmation(undefined);
                        void act("apply_reference");
                      }}
                    >
                      Apply bundled reference data
                    </Button>
                    <Button onClick={() => setConfirmation(undefined)}>
                      Keep current data
                    </Button>
                  </Space>
                }
              />
            ) : pendingPin ? (
              <Button
                type="primary"
                disabled={sending}
                onClick={() => setConfirmation(setup.sequence)}
              >
                Apply bundled reference data
              </Button>
            ) : (
              <Button disabled={sending} onClick={() => void act("check_reference")}>
                Check and repair reference data
              </Button>
            ))}
        </Space>
      </AtlasDrawer>
    </>
  );
}
