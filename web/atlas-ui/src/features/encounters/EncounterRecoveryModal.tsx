import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button, Checkbox, InputNumber, Space, Spin } from "antd";
import { EncounterModal } from "./EncounterModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { previewEncounterRecovery } from "../../api/atlasApi";
import type {
  EncounterDetailView,
  EncounterRecoveryAction,
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
  const [snapshot] = useState(encounter);
  const rest = mode === "rest_overnight";
  const candidates = snapshot.participants.filter(
    (p) => p.participant_kind !== "hazard" && p.mortality !== "dead",
  );
  const [chosen, setChosen] = useState(() =>
    candidates
      .filter((p) => p.side === "pc" || p.side === "ally")
      .map((p) => p.participant_key),
  );
  const [healing, setHealing] = useState<Record<string, bigint | null>>({});
  const [submitError, setSubmitError] = useState<unknown>(null);
  const action: EncounterRecoveryAction = rest
    ? {
        kind: "rest_overnight",
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
    retry: false,
  });
  const busy = mutations.pending;
  const stale = encounter.revision !== snapshot.revision;
  const resetName =
    typeof mode === "object"
      ? snapshot.participants.find((p) => p.participant_key === mode.participantKey)
          ?.display_name
      : undefined;
  return (
    <EncounterModal
      open
      title={
        rest ? "Rest overnight" : resetName ? `Reset ${resetName}` : "Reset encounter"
      }
      width={640}
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
            danger={!rest}
            loading={busy}
            disabled={
              stale ||
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
      {!rest && (
        <p>
          Restore creation HP, initiative, variant, conditions and resources. Names,
          notes, sides and visibility are kept.
        </p>
      )}
      {rest && (
        <p>
          Apply 8 hours of rest and daily preparations to the selected participants.
        </p>
      )}
      {stale && (
        <p role="alert">
          Encounter changed. Close this review and reopen it to use the current values.
        </p>
      )}
      {submitError != null && (
        <ErrorNotice error={submitError} title="Recovery could not save" />
      )}
      {preview.error && (
        <ErrorNotice
          error={preview.error}
          title="Recovery preview could not load"
          action={<Button onClick={() => void preview.refetch()}>Retry preview</Button>}
        />
      )}
      {rest && candidates.length === 0 && <p>No living participants can rest.</p>}
      {rest && chosen.length === 0 && candidates.length > 0 && (
        <p>Choose participants to rest.</p>
      )}
      <div className="encounter-recovery__participants" aria-busy={preview.isFetching}>
        {(rest
          ? candidates
          : snapshot.participants.filter(
              (p) =>
                mode === "reset_encounter" ||
                (typeof mode === "object" && p.participant_key === mode.participantKey),
            )
        ).map((p) => {
          const row = preview.data?.participants.find(
            (r) => r.participant_key === p.participant_key,
          );
          const selected = !rest || chosen.includes(p.participant_key);
          return (
            <section
              className="encounter-recovery__participant"
              key={p.participant_key}
            >
              <div className="encounter-recovery__identity">
                {rest ? (
                  <Checkbox
                    checked={selected}
                    disabled={busy}
                    onChange={(event) => {
                      setChosen((keys) =>
                        event.target.checked
                          ? [...keys, p.participant_key]
                          : keys.filter((k) => k !== p.participant_key),
                      );
                    }}
                  >
                    {p.display_name}
                  </Checkbox>
                ) : (
                  <strong>{p.display_name}</strong>
                )}
                {rest && selected && (
                  <label className="encounter-recovery__healing">
                    HP recovery
                    <InputNumber
                      stringMode
                      min="0"
                      max="9007199254740991"
                      precision={0}
                      disabled={busy}
                      aria-label={`HP recovery for ${p.display_name}`}
                      placeholder={row?.healing?.toString() ?? "Enter amount"}
                      value={healing[p.participant_key]?.toString() ?? null}
                      onChange={(value) => {
                        if (value == null || /^\d+$/.test(String(value)))
                          setHealing((values) => ({
                            ...values,
                            [p.participant_key]: value == null ? null : BigInt(value),
                          }));
                      }}
                    />
                  </label>
                )}
              </div>
              {selected && (
                <div className="encounter-recovery__consequences">
                  <div>
                    HP {p.current_hp?.toString() ?? "—"} →{" "}
                    {row?.resulting_hp?.toString() ?? (preview.isFetching ? "…" : "—")}
                  </div>
                  {row?.unavailable_reason && (
                    <p role="status">{row.unavailable_reason}</p>
                  )}
                  {!!row?.changes.length && (
                    <ul>
                      {row.changes.map((change, index) => (
                        <li key={index}>{change}</li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </section>
          );
        })}
      </div>
      {preview.isFetching && (
        <Spin size="small" aria-label="Updating recovery preview" />
      )}
      {preview.data?.changes.map((change) => (
        <p key={change}>{change}</p>
      ))}
    </EncounterModal>
  );
}
