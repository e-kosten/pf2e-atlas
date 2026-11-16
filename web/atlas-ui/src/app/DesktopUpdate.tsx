import { Alert, Button, Flex, Progress, Typography } from "antd";
import { ArrowDownToLine } from "lucide-react";
import { useState, useSyncExternalStore } from "react";
import { AtlasModal } from "../shared/ui/overlays/AtlasModal";

type Phase =
  | { kind: "available" }
  | { kind: "downloading"; percent: number | null }
  | { kind: "installing" };

type UpdateAttachment = {
  token: string;
  update?: { version: string; phase: Phase; error: string | null } | null;
};

type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: UpdateAttachment;
  __TAURI__?: {
    core: {
      invoke: (
        command: "desktop_update_action",
        args: { action: "details" | "install"; token: string; version: string },
      ) => Promise<string | null>;
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
