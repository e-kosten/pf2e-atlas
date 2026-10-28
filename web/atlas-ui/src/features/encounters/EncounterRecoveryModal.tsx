import { useId, useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { Button, InputNumber, Radio, Space, Spin, Table } from "antd";
import type { TableColumnsType } from "antd";
import { EncounterModal } from "./EncounterModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { previewEncounterRecovery } from "../../api/atlasApi";
import type {
  EncounterDetailView,
  EncounterParticipantView,
  EncounterRecoveryAction,
  OvernightHealingMode,
} from "../../generated/atlas";
import type { useEncounterMutations } from "./useEncounterMutations";

type Mode = "reset_encounter" | "rest_overnight" | { participantKey: string };
export function EncounterRecoveryModal({
  encounter,
  mode,
  mutations,
  onClose,
}: {
  encounter: Pick<EncounterDetailView, "participants" | "revision" | "encounter">;
  mode: Mode;
  mutations: Pick<ReturnType<typeof useEncounterMutations>, "run" | "pending">;
  onClose: () => void;
}) {
  const reasonId = useId();
  const [snapshot] = useState(encounter);
  const rest = mode === "rest_overnight";
  const candidates = snapshot.participants.filter(
    (p) => p.participant_kind !== "hazard" && p.mortality !== "dead",
  );
  const [chosen, setChosen] = useState(() => candidates.map((p) => p.participant_key));
  const [healingMode, setHealingMode] = useState<OvernightHealingMode>("natural");
  const [healing, setHealing] = useState<Record<string, bigint | null>>({});
  const [submitError, setSubmitError] = useState<unknown>(null);
  const action: EncounterRecoveryAction = rest
    ? {
        kind: "rest_overnight",
        healing_mode: healingMode,
        participants: chosen.map((participant_key) => ({
          participant_key,
          healing: healing[participant_key] ?? null,
        })),
      }
    : mode === "reset_encounter"
      ? { kind: "reset_encounter" }
      : {
          kind: "reset_participant",
          participant_key: typeof mode === "object" ? mode.participantKey : "",
        };
  const request = { expected_encounter_revision: snapshot.revision, action };
  const identity = JSON.stringify(request, (_key, value) =>
    typeof value === "bigint" ? value.toString() : value,
  );
  const preview = useQuery({
    queryKey: [
      "encounter-recovery-preview",
      snapshot.encounter.encounter_key,
      identity,
    ],
    queryFn: ({ signal }) =>
      previewEncounterRecovery(snapshot.encounter.encounter_key, request, signal),
    enabled: !rest || chosen.length > 0,
    // The creation snapshot and participant identities stay fixed for this review.
    // Retain their last preview while changing healing inputs; submission waits.
    placeholderData: keepPreviousData,
    retry: false,
  });
  const busy = mutations.pending;
  const stale = encounter.revision !== snapshot.revision;
  const resetName =
    typeof mode === "object"
      ? snapshot.participants.find((p) => p.participant_key === mode.participantKey)
          ?.display_name
      : undefined;
  const participants = rest
    ? candidates
    : snapshot.participants.filter(
        (p) =>
          mode === "reset_encounter" ||
          (typeof mode === "object" && p.participant_key === mode.participantKey),
      );
  const rowFor = (p: EncounterParticipantView) =>
    preview.data?.participants.find((r) => r.participant_key === p.participant_key);
  const selected = (p: EncounterParticipantView) =>
    !rest || chosen.includes(p.participant_key);
  const missingAmount = "Enter the missing HP recovery amounts.";
  const blankDraft = (key: string) =>
    healingMode === "natural" &&
    Object.prototype.hasOwnProperty.call(healing, key) &&
    healing[key] === null;
  const reasonFor = (p: EncounterParticipantView) =>
    !selected(p)
      ? null
      : blankDraft(p.participant_key)
        ? missingAmount
        : rowFor(p)?.unavailable_reason;
  const reasons = [
    ...new Set(participants.flatMap((p) => (reasonFor(p) ? [reasonFor(p)!] : []))),
  ];
  const hasBlankDraft = rest && chosen.some(blankDraft);
  const changesFor = (p: EncounterParticipantView) => {
    const row = rowFor(p);
    if (!selected(p)) return null;
    return row?.changes.length ? (
      <div className="encounter-recovery__changes">
        {row.changes.map((change, index) => (
          <div key={index}>{change}</div>
        ))}
      </div>
    ) : row && !row.unavailable_reason ? (
      <span className="muted">—</span>
    ) : null;
  };
  const columns: TableColumnsType<EncounterParticipantView> = [
    {
      title: "Participant",
      key: "participant",
      render: (_value, p) => (
        <div>
          <strong>{p.display_name}</strong>
          <div className="encounter-recovery__compact-changes">{changesFor(p)}</div>
        </div>
      ),
    },
    {
      title: "HP",
      key: "hp",
      width: 140,
      render: (_value, p) => {
        const row = rowFor(p);
        const active = selected(p);
        return (
          <div className="encounter-recovery__hp">
            <div className="encounter-recovery__hp-result">
              {p.current_hp?.toString() ?? "—"}
              {active && (
                <>
                  {" "}
                  → <strong>{row?.resulting_hp?.toString() ?? "…"}</strong>
                </>
              )}
              {rest && active && row?.resulting_max_hp != null && (
                <span className="muted"> / {row.resulting_max_hp.toString()}</span>
              )}
            </div>
            {rest && active && healingMode === "natural" && (
              <InputNumber
                className="encounter-recovery__amount"
                stringMode
                min="0"
                max="9007199254740991"
                precision={0}
                controls={false}
                disabled={busy}
                prefix="+"
                suffix="HP"
                aria-label={`HP recovery for ${p.display_name}`}
                placeholder="Amount"
                value={
                  Object.prototype.hasOwnProperty.call(healing, p.participant_key)
                    ? (healing[p.participant_key]?.toString() ?? null)
                    : (row?.healing?.toString() ?? null)
                }
                status={reasonFor(p) ? "error" : undefined}
                aria-invalid={!!reasonFor(p)}
                aria-describedby={
                  reasonFor(p)
                    ? `${reasonId}-${reasons.indexOf(reasonFor(p)!)}`
                    : undefined
                }
                onChange={(value) => {
                  if (value == null || /^\d+$/.test(String(value)))
                    setHealing((values) => ({
                      ...values,
                      [p.participant_key]: value == null ? null : BigInt(value),
                    }));
                }}
              />
            )}
          </div>
        );
      },
    },
    {
      title: "Other changes",
      key: "changes",
      responsive: ["md"],
      render: (_value, p) => changesFor(p),
    },
  ];
  return (
    <EncounterModal
      open
      title={
        rest ? "Rest overnight" : resetName ? `Reset ${resetName}` : "Reset encounter"
      }
      width={760}
      onCancel={onClose}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: !busy }}
      footer={
        <Space>
          <Button disabled={busy} onClick={onClose}>
            Cancel
          </Button>
          <Button
            type="primary"
            loading={busy}
            disabled={
              stale ||
              hasBlankDraft ||
              preview.isFetching ||
              !preview.data?.can_apply ||
              (rest && chosen.length === 0)
            }
            onClick={async () => {
              setSubmitError(null);
              try {
                const result = await mutations.run({ kind: "recovery", request });
                if (result.active) onClose();
              } catch (error) {
                setSubmitError(error);
              }
            }}
          >
            {rest
              ? "Rest overnight"
              : resetName
                ? "Reset participant"
                : "Reset encounter"}
          </Button>
        </Space>
      }
    >
      <div className="encounter-recovery">
        <p className="encounter-recovery__intro">
          {rest
            ? "8 hours of rest and daily preparations."
            : "Restore HP, initiative, adjustments, conditions and resources to their creation values."}
        </p>
        {rest && (
          <div className="encounter-recovery__policy">
            <span id="recovery-hp-label">HP recovery</span>
            <Radio.Group
              aria-labelledby="recovery-hp-label"
              value={healingMode}
              disabled={busy}
              onChange={(event) =>
                setHealingMode(event.target.value as OvernightHealingMode)
              }
              options={[
                { label: "Overnight", value: "natural" },
                { label: "Full HP", value: "full" },
              ]}
            />
          </div>
        )}
        {stale && (
          <p role="alert">
            Encounter changed. Close this review and reopen it to use the current
            values.
          </p>
        )}
        {submitError != null && (
          <ErrorNotice error={submitError} title="Recovery could not save" />
        )}
        {preview.error && (
          <ErrorNotice
            error={preview.error}
            title="Recovery preview could not load"
            action={
              <Button onClick={() => void preview.refetch()}>Retry preview</Button>
            }
          />
        )}
        {reasons.map((reason, index) => (
          <p
            id={`${reasonId}-${index}`}
            className="encounter-recovery__notice"
            role="status"
            key={reason}
          >
            {reason}
          </p>
        ))}
        {rest && candidates.length === 0 ? (
          <p>No living participants can rest.</p>
        ) : (
          <Table
            className="encounter-recovery__table"
            size="small"
            tableLayout="fixed"
            pagination={false}
            rowKey="participant_key"
            dataSource={participants}
            columns={columns}
            rowSelection={
              rest
                ? {
                    selectedRowKeys: chosen,
                    onChange: (keys) => setChosen(keys.map(String)),
                    getCheckboxProps: (p) => ({
                      disabled: busy,
                      "aria-label": `Rest ${p.display_name}`,
                    }),
                  }
                : undefined
            }
          />
        )}
        <div className="encounter-recovery__summary" aria-busy={preview.isFetching}>
          {preview.isFetching && (
            <Spin size="small" aria-label="Updating recovery preview" />
          )}
          {rest && chosen.length === 0 ? (
            <span>Choose participants to rest.</span>
          ) : (
            preview.data?.changes.map((change) => <div key={change}>{change}</div>)
          )}
          {!rest && (
            <div className="muted">Names, notes, sides and visibility are kept.</div>
          )}
        </div>
      </div>
    </EncounterModal>
  );
}
