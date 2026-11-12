import {
  Alert,
  Button,
  Checkbox,
  Progress,
  Select,
  Space,
  Tooltip,
  Typography,
} from "antd";
import { Settings } from "lucide-react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { restoreOverlayFocus } from "../shared/ui/overlays/restoreOverlayFocus";
import { AtlasDrawer } from "../shared/ui/overlays/AtlasDrawer";

type SetupStatus = "idle" | "available" | "running" | "cancelling" | "failed";
type SetupAction =
  | "cancel"
  | "resume"
  | "build_locally"
  | "apply_reference"
  | "check_reference"
  | "read_reference_status"
  | "apply_models"
  | "enable_terminal";
type ModelSelection = { profiles: string[]; preferred: string | null };
type ModelStatus = { id: string; label: string; installed: boolean; usable: boolean };
type ReferenceStatus = {
  revision: string | null;
  target: string;
  targetVersion: string;
  managed: boolean;
  available: boolean;
  modelsAvailable?: boolean;
  canChangeModels?: boolean;
  models?: ModelStatus[];
  preferred?: string | null;
};
type DesktopAttachment = {
  token: string;
  openRequest?: number | null;
  referenceLoading?: boolean;
  reference?: ReferenceStatus | null;
  setup: {
    job: number;
    sequence: number;
    status: SetupStatus;
    progress?: { message?: string; current?: number; total?: number; unit?: string };
    message?: string;
    local_build: boolean;
    can_resume: boolean;
    reference_job?: boolean;
    model_job?: boolean;
  };
};
type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: DesktopAttachment;
  __TAURI__?: {
    core: { invoke: (command: string, args: object) => Promise<string | null> };
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
  const [terminalPath, setTerminalPath] = useState<string | null>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
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

  async function act(action: SetupAction, modelSelection?: ModelSelection) {
    const host = (window as DesktopWindow).__TAURI__;
    if (!host || sending || !attachment) return;
    setError(null);
    setSending(true);
    try {
      const result = await host.core.invoke("desktop_setup_action", {
        action,
        token: attachment.token,
        job: attachment.setup.job,
        sequence: attachment.setup.sequence,
        ...(modelSelection ? { modelSelection } : {}),
      });
      if (action === "enable_terminal") setTerminalPath(result);
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
      <Tooltip
        title="Setup and Updates"
        trigger={["hover", "focus"]}
        open={open ? false : undefined}
      >
        <Button
          ref={trigger}
          type="text"
          aria-label="Setup and Updates"
          icon={<Settings size={18} />}
          onClick={() => {
            setConfirmation(undefined);
            setOpen(true);
            if (!running) void act("read_reference_status");
          }}
        />
      </Tooltip>
      <AtlasDrawer
        title="Setup and Updates"
        open={open}
        onClose={() => {
          restoreOverlayFocus(trigger.current, panel.current);
          setOpen(false);
        }}
      >
        <div ref={panel}>
          <Space orientation="vertical" size="middle">
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
              <Typography.Text role="status">
                Checking reference status…
              </Typography.Text>
            )}
            {!running && !referenceLoading && !reference && !setup.can_resume && (
              <Button
                loading={sending}
                onClick={() => void act("read_reference_status")}
              >
                Check reference status
              </Button>
            )}
            {running && (
              <>
                <Typography.Text role="status">
                  {status === "cancelling"
                    ? "Cancelling setup…"
                    : (progress?.message ??
                      (setup.reference_job && !setup.model_job
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
              <Button
                type="primary"
                loading={sending}
                onClick={() => void act("resume")}
              >
                {status === "failed" ? "Retry setup" : "Resume semantic setup"}
              </Button>
            )}
            {setup.local_build &&
              status === "failed" &&
              (setup.can_resume || reference?.available || setup.model_job) &&
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
            {reference?.models && (
              <SearchModels
                key={JSON.stringify([
                  reference.models.map((model) => [model.id, model.installed]),
                  reference.preferred,
                ])}
                reference={reference}
                disabled={running || sending || !!referenceLoading || setup.can_resume}
                onApply={(selection) => void act("apply_models", selection)}
              />
            )}
            <Space orientation="vertical" size="small">
              <Typography.Text strong>Terminal and agent access</Typography.Text>
              <Typography.Text>
                Link the bundled Atlas command at <code>~/.local/bin/atlas</code>.
              </Typography.Text>
              <Button disabled={sending} onClick={() => void act("enable_terminal")}>
                Enable terminal command
              </Button>
              {terminalPath && (
                <Typography.Text role="status">
                  Linked at <code>{terminalPath}</code>. Run{" "}
                  <code>command -v atlas</code> to check which command your shell uses.
                </Typography.Text>
              )}
            </Space>
          </Space>
        </div>
      </AtlasDrawer>
    </>
  );
}

function SearchModels({
  reference,
  disabled,
  onApply,
}: {
  reference: ReferenceStatus;
  disabled: boolean;
  onApply: (selection: ModelSelection) => void;
}) {
  const models = reference.models ?? [];
  const installed = models
    .filter((model) => model.installed)
    .map((model) => model.id)
    .sort();
  const [profiles, setProfiles] = useState(installed);
  const [preferred, setPreferred] = useState(reference.preferred ?? "");
  const membershipChanged = profiles.join(",") !== installed.join(",");
  const dirty = membershipChanged || preferred !== (reference.preferred ?? "");
  const editable = !disabled && !!reference.modelsAvailable;
  return (
    <Space orientation="vertical" size="small">
      <Typography.Text strong>Search models</Typography.Text>
      <Checkbox.Group
        aria-label="Installed search models"
        value={profiles}
        disabled={!editable || !reference.canChangeModels}
        options={models.map((model) => ({
          value: model.id,
          label:
            model.installed && !model.usable
              ? `${model.label} (needs repair)`
              : model.label,
        }))}
        onChange={(next) => {
          const selected = next.sort();
          setProfiles(selected);
          if (!selected.includes(preferred)) setPreferred("");
        }}
      />
      <Typography.Text>
        {profiles.length === 0
          ? "Keyword-only search uses no model. Similar records requires a search model."
          : "Adding a model may download files and rebuild parts of the search index. Editing pauses while the index is prepared."}
      </Typography.Text>
      {reference.modelsAvailable && !reference.canChangeModels && (
        <Typography.Text type="secondary">
          Installed models can be changed after the workspace uses the bundled reference
          data.
        </Typography.Text>
      )}
      <Space orientation="vertical" size="small">
        <Typography.Text>Default search model</Typography.Text>
        <Select
          aria-label="Default search model"
          value={preferred}
          disabled={!editable || profiles.length === 0}
          options={[
            { value: "", label: "Automatic" },
            ...models
              .filter((model) => profiles.includes(model.id))
              .map((model) => ({
                value: model.id,
                label: model.label,
                disabled: !membershipChanged && !model.usable,
              })),
          ]}
          onChange={setPreferred}
        />
      </Space>
      <Button
        type="primary"
        disabled={
          !editable || !dirty || (membershipChanged && !reference.canChangeModels)
        }
        onClick={() => onApply({ profiles, preferred: preferred || null })}
      >
        Apply search models
      </Button>
    </Space>
  );
}
