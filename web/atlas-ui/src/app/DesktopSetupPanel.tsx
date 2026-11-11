import { Alert, Button, Drawer, Progress, Space, Typography } from "antd";
import { useState, useSyncExternalStore } from "react";

type SetupStatus = "idle" | "available" | "running" | "cancelling" | "failed";
type SetupAction = "cancel" | "resume" | "build_locally";
type DesktopAttachment = {
  token: string;
  setup: {
    job: number;
    status: SetupStatus;
    progress?: { message?: string; current?: number; total?: number; unit?: string };
    message?: string;
    local_build: boolean;
    can_resume: boolean;
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
  const setup = attachment?.setup;
  if (!attachment || !setup || (setup.status === "idle" && !open)) return null;
  const { status, progress } = setup;
  const running = status === "running" || status === "cancelling";
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
      <Button type="text" onClick={() => setOpen(true)}>
        {running ? "Semantic setup in progress" : "Semantic setup"}
      </Button>
      <Drawer title="Semantic search setup" open={open} onClose={() => setOpen(false)}>
        <Space vertical size="middle">
          {running && (
            <>
              <Typography.Text role="status">
                {status === "cancelling"
                  ? "Cancelling setup…"
                  : (progress?.message ?? "Preparing semantic search…")}
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
          {status === "idle" && (
            <Typography.Text>
              {setup.message ?? "Semantic search is ready."}
            </Typography.Text>
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
          {setup.can_resume && setup.local_build && status === "failed" && (
            <Button disabled={sending} onClick={() => void act("build_locally")}>
              Build embeddings locally
            </Button>
          )}
        </Space>
      </Drawer>
    </>
  );
}
