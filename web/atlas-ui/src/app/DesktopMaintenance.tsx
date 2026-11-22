import { Alert, Button, Checkbox, Progress, Select, Space, Typography } from "antd";
import { useEffect, useState } from "react";
import {
  invokeDesktop,
  useDesktopAttachment,
  type ModelSelection,
  type ReferenceStatus,
} from "./desktopBridge";
import { useSavedWritesPaused } from "../shared/feedback/SavedWritesPaused";
import { useTabCloseGuard } from "./DesktopTabContext";

type SetupAction =
  | "cancel"
  | "resume"
  | "build_locally"
  | "apply_reference"
  | "check_reference"
  | "read_reference_status"
  | "apply_models";
export function DesktopSearchData() {
  const attachment = useDesktopAttachment();
  const writesPaused = useSavedWritesPaused();
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [confirmation, setConfirmation] = useState<number>();
  const setup = attachment?.setup;
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
    if (sending || !attachment) return;
    setError(null);
    setSending(true);
    try {
      await invokeDesktop("desktop_setup_action", {
        action,
        token: attachment.token,
        job: attachment.setup.job,
        sequence: attachment.setup.sequence,
        ...(modelSelection ? { modelSelection } : {}),
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
    <div className="desktop-setup">
      {(running ||
        setup.message ||
        setup.can_resume ||
        error ||
        (setup.local_build && status === "failed")) && (
        <section
          className="desktop-setup__section"
          aria-labelledby="desktop-setup-activity"
        >
          <h2 id="desktop-setup-activity">Setup activity</h2>
          {running && (
            <div className="desktop-setup__stack">
              <Typography.Text role="status">
                {status === "cancelling"
                  ? "Cancelling setup…"
                  : (progress?.message ??
                    (setup.reference_job && !setup.model_job
                      ? "Preparing reference data…"
                      : "Preparing semantic search…"))}
              </Typography.Text>
              {percent !== undefined && (
                <Progress
                  percent={percent}
                  status="normal"
                  aria-label="Setup progress"
                />
              )}
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
            </div>
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
              disabled={writesPaused}
              onClick={() => void act("resume")}
            >
              {status === "failed" ? "Retry setup" : "Resume semantic setup"}
            </Button>
          )}
          {setup.local_build &&
            status === "failed" &&
            (setup.can_resume || reference?.available || setup.model_job) &&
            !referenceLoading && (
              <>
                <Typography.Text type="secondary">
                  Building locally can take an hour or more.
                </Typography.Text>
                <Button
                  disabled={sending || writesPaused}
                  onClick={() => void act("build_locally")}
                >
                  Build embeddings locally
                </Button>
              </>
            )}
        </section>
      )}
      <section
        className="desktop-setup__section"
        aria-labelledby="desktop-setup-reference"
      >
        <h2 id="desktop-setup-reference">Reference data</h2>
        {reference && (
          <div className="desktop-setup__stack">
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
          </div>
        )}
        {!running && referenceLoading && (
          <Typography.Text role="status">Checking reference status…</Typography.Text>
        )}
        {!running && !referenceLoading && !reference && !setup.can_resume && (
          <Button loading={sending} onClick={() => void act("read_reference_status")}>
            Check reference status
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
      </section>
      {reference?.models && (
        <SearchModels
          key={JSON.stringify([
            reference.revision,
            reference.target,
            reference.models.map((model) => [model.id, model.installed, model.usable]),
            reference.preferred,
          ])}
          reference={reference}
          disabled={running || sending || !!referenceLoading || setup.can_resume}
          onApply={(selection) => void act("apply_models", selection)}
        />
      )}
    </div>
  );
}

type MacTerminalStatus = {
  state: "missing" | "linked" | "replaceable" | "conflict";
  path: string;
  version: string;
};

type TerminalStatus = MacTerminalStatus | { command: string };
type TerminalAction =
  "read_terminal" | "enable_terminal" | "switch_terminal" | "choose_terminal_folder";

export function DesktopTerminal() {
  const attachment = useDesktopAttachment();
  if (!attachment) return null;
  return (
    <TerminalAccess
      key={attachment.token}
      token={attachment.token}
      job={attachment.setup.job}
      workspace={attachment.channel?.workspace_path}
    />
  );
}

function TerminalAccess({
  token,
  job,
  workspace,
}: {
  token: string;
  job: number;
  workspace?: string | null;
}) {
  const [status, setStatus] = useState<TerminalStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [customLink, setCustomLink] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let active = true;
    void invokeDesktop<TerminalStatus>("desktop_setup_action", {
      action: "read_terminal",
      token,
      job,
    })
      .then((result) => {
        if (active) setStatus(result);
      })
      .catch((failure) => {
        if (active) setError(terminalError(failure));
      })
      .finally(() => {
        if (active) setBusy(false);
      });
    return () => {
      active = false;
    };
  }, [token, job]);

  async function change(action: TerminalAction) {
    if (busy) return;
    setBusy(true);
    setError(null);
    setCustomLink(null);
    try {
      const result = await invokeDesktop<TerminalStatus | null>(
        "desktop_setup_action",
        { action, token, job },
      );
      if (action === "choose_terminal_folder") {
        if (result && "path" in result) setCustomLink(result.path);
      } else if (result) {
        setStatus(result);
      }
    } catch (failure) {
      setError(terminalError(failure));
      if (action !== "read_terminal") {
        setStatus(
          await invokeDesktop<TerminalStatus>("desktop_setup_action", {
            action: "read_terminal",
            token,
            job,
          }).catch(() => null),
        );
      }
    } finally {
      setBusy(false);
    }
  }

  async function copyCommand(command: string) {
    setError(null);
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
    } catch {
      setCopied(false);
      setError("Couldn’t copy the command. Select the command and copy it manually.");
    }
  }

  return (
    <section className="desktop-setup__section">
      {!status && busy && (
        <Typography.Text role="status">Checking terminal command…</Typography.Text>
      )}
      {error && <Alert type="error" title={error} />}
      {!status && !busy && (
        <Button onClick={() => void change("read_terminal")}>Retry</Button>
      )}
      {status &&
        ("command" in status ? (
          <>
            <h2>Run Atlas in PowerShell</h2>
            <code className="desktop-settings__command">{status.command}</code>
            <Button onClick={() => void copyCommand(status.command)}>
              Copy command
            </Button>
            {copied && <Typography.Text role="status">Command copied.</Typography.Text>}
          </>
        ) : (
          <>
            <h2>Terminal command</h2>
            {status.state === "missing" && (
              <Button loading={busy} onClick={() => void change("enable_terminal")}>
                Install terminal command
              </Button>
            )}
            {status.state === "replaceable" && (
              <>
                <Typography.Text>An Atlas command already exists here.</Typography.Text>
                <Button loading={busy} onClick={() => void change("switch_terminal")}>
                  Switch to bundled CLI
                </Button>
              </>
            )}
            {status.state === "conflict" && (
              <Alert
                type="warning"
                title={
                  <>
                    An entry already exists at <code>{status.path}</code>. Move it to
                    enable the bundled command here.
                  </>
                }
              />
            )}
            {status.state === "linked" && (
              <Typography.Text role="status">
                The terminal command is installed.
              </Typography.Text>
            )}
            <details className="desktop-setup__terminal-details">
              <summary>Command details</summary>
              <dl>
                <dt>Bundled version</dt>
                <dd>{status.version}</dd>
                <dt>Command path</dt>
                <dd>
                  <code>{status.path}</code>
                </dd>
                {workspace && (
                  <>
                    <dt>Workspace</dt>
                    <dd>
                      <code>{workspace}</code>
                    </dd>
                  </>
                )}
              </dl>
              {status.state === "linked" && (
                <p>
                  Run <code>command -v atlas</code> to check which command your shell
                  uses.
                </p>
              )}
              <Button
                disabled={busy}
                onClick={() => void change("choose_terminal_folder")}
              >
                Choose bin folder…
              </Button>
              {customLink && (
                <p role="status">
                  Command linked at <code>{customLink}</code>.
                </p>
              )}
            </details>
          </>
        ))}
    </section>
  );
}
function terminalError(error: unknown) {
  return typeof error === "string"
    ? error
    : "Atlas could not check or change the terminal command.";
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
  const installedModels = models.filter((model) => model.installed);
  const installed = installedModels.map((model) => model.id).sort();
  const [profiles, setProfiles] = useState(installed);
  const [preferred, setPreferred] = useState(reference.preferred ?? "");
  const membershipChanged = profiles.join(",") !== installed.join(",");
  const dirty = membershipChanged || preferred !== (reference.preferred ?? "");
  useTabCloseGuard(() => (dirty ? "dirty" : undefined));
  const editable = !disabled && !!reference.modelsAvailable;
  const canEditMembership = editable && !!reference.canChangeModels;
  return (
    <section className="desktop-setup__section" aria-labelledby="desktop-setup-models">
      <h2 id="desktop-setup-models">Search models</h2>
      {canEditMembership ? (
        <Checkbox.Group
          aria-label="Installed search models"
          className="desktop-setup__model-options"
          value={profiles}
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
      ) : (
        installedModels.length > 0 && (
          <ul className="desktop-setup__model-status">
            {installedModels.map((model) => (
              <li key={model.id}>
                <span>{model.label}</span>
                <Typography.Text type="secondary">
                  {model.usable ? "Installed" : "Needs repair"}
                </Typography.Text>
              </li>
            ))}
          </ul>
        )
      )}
      {reference.modelsAvailable && !reference.canChangeModels && (
        <Typography.Text type="secondary">
          {!reference.managed || !reference.available
            ? "Use the Atlas CLI to add or remove models for this workspace."
            : reference.revision !== reference.target
              ? "Apply the bundled reference data above to add or remove models."
              : "Check and repair the reference data before adding or removing models."}
        </Typography.Text>
      )}
      <div className="desktop-setup__stack desktop-setup__model-preference">
        <Typography.Text>Preferred search model</Typography.Text>
        <Select
          aria-label="Preferred search model"
          value={preferred}
          popupMatchSelectWidth={false}
          disabled={!editable || profiles.length === 0}
          options={[
            { value: "", label: "Atlas default" },
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
      </div>
      {(canEditMembership ? profiles.length : installed.length) === 0 && (
        <Typography.Text type="secondary">
          Keyword-only search. Similar records requires a search model.
        </Typography.Text>
      )}
      {canEditMembership && membershipChanged && (
        <Typography.Text type="secondary">
          Model changes may download files and rebuild parts of the search index.
          Editing pauses while the index is prepared.
        </Typography.Text>
      )}
      {dirty && (
        <Button
          type="primary"
          disabled={!editable || (membershipChanged && !canEditMembership)}
          onClick={() => onApply({ profiles, preferred: preferred || null })}
        >
          Apply
        </Button>
      )}
    </section>
  );
}
