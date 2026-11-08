import type { FormInstance } from "antd";
import { useLayoutEffect, useRef, useState } from "react";
import type {
  EncounterDetailView,
  EncounterStatusView,
  EncounterSummaryView,
  UpdateEncounterRequest,
} from "../../generated/atlas";
import { savedEditConflictFields } from "../../shared/feedback/savedEditConflict";
import { encounterStatusLabels } from "./encounterStatus";

type SavedSettings = {
  name: string;
  description: string | null;
  note: string | null;
  status: EncounterStatusView;
  round_duration_seconds: number;
};

export type EncounterSettingsValues = Omit<
  SavedSettings,
  "description" | "note" | "round_duration_seconds"
> & {
  description: string;
  note: string;
  round_duration_seconds: number;
};

type Setting = keyof SavedSettings;
type Conflict = {
  fields: Setting[];
  saved?: SavedSettings;
  loading: boolean;
};

const labels = {
  name: "Name",
  description: "Description",
  note: "Note",
  status: "Status",
  round_duration_seconds: "Seconds per round",
} satisfies Record<Setting, string>;

const fields = Object.keys(labels) as Setting[];

function isSetting(field: string): field is Setting {
  return fields.some((setting) => setting === field);
}

function savedSettings(
  encounter: EncounterSummaryView,
  note: string | undefined,
): SavedSettings {
  return {
    name: encounter.name,
    description: encounter.description ?? null,
    note: note ?? null,
    status: encounter.status,
    round_duration_seconds: encounter.round_duration_seconds,
  };
}

function formValues(saved: SavedSettings): EncounterSettingsValues {
  return {
    ...saved,
    description: saved.description ?? "",
    note: saved.note ?? "",
    round_duration_seconds: saved.round_duration_seconds,
  };
}

function describe(setting: Setting, value: SavedSettings[Setting] | number): string {
  if (setting === "status") return encounterStatusLabels[value as EncounterStatusView];
  return value == null || value === "" ? "Empty" : String(value);
}

function copySetting<K extends Setting>(
  target: SavedSettings,
  source: SavedSettings,
  field: K,
) {
  target[field] = source[field];
}

export function useEncounterSettingsDraft({
  encounter,
  encounterNote,
  form,
  onSave,
  onRefreshConflict,
  onConflictChange,
}: {
  encounter: EncounterSummaryView;
  encounterNote: string | undefined;
  form: FormInstance<EncounterSettingsValues>;
  onSave: (request: UpdateEncounterRequest) => Promise<unknown>;
  onRefreshConflict: () => Promise<EncounterDetailView | undefined>;
  onConflictChange?: (blocked: boolean) => void;
}) {
  const baseline = useRef(savedSettings(encounter, encounterNote));
  const [conflict, setConflict] = useState<Conflict>();

  useLayoutEffect(() => {
    const saved = savedSettings(encounter, encounterNote);
    const values = formValues(saved);
    // Advance the expected value only when the corresponding input is updated.
    const untouched = fields.filter((field) => !form.isFieldTouched(field));
    for (const field of untouched) {
      copySetting(baseline.current, saved, field);
    }
    form.setFields(
      untouched.map((name) => ({ name, value: values[name], touched: false })),
    );
  }, [encounter, encounterNote, form]);

  async function loadConflict(conflictingFields: Setting[]) {
    setConflict({ fields: conflictingFields, loading: true });
    onConflictChange?.(true);
    const view = await onRefreshConflict();
    setConflict({
      fields: conflictingFields,
      saved:
        view?.encounter.encounter_key === encounter.encounter_key
          ? savedSettings(view.encounter, view.note)
          : undefined,
      loading: false,
    });
  }

  async function submit(values: EncounterSettingsValues) {
    if (conflict) return;
    const expected = baseline.current;
    const original = formValues(expected);
    function edit<K extends Setting>(
      field: K,
      encode: (value: EncounterSettingsValues[K]) => SavedSettings[K],
    ) {
      if (!form.isFieldTouched(field) || original[field] === values[field]) {
        return undefined;
      }
      return { expected: expected[field], value: encode(values[field]) };
    }

    const name = edit("name", (value) => value);
    const description = edit("description", (value) => value || null);
    const note = edit("note", (value) => value || null);
    const status = edit("status", (value) => value);
    const roundDuration = edit("round_duration_seconds", (value) => value);
    try {
      await onSave({
        encounter_key: encounter.encounter_key,
        ...(name && { name }),
        ...(description && { description }),
        ...(note && { note }),
        ...(status && { status }),
        ...(roundDuration && { round_duration_seconds: roundDuration }),
      });
      form.setFields(fields.map((field) => ({ name: field, touched: false })));
    } catch (error) {
      const conflictingFields = savedEditConflictFields(error)?.filter(isSetting);
      if (conflictingFields?.length) await loadConflict(conflictingFields);
      // Other failures belong to the encounter mutation owner.
    }
  }

  function resolve(field: Setting, saved: SavedSettings, useSaved: boolean) {
    copySetting(baseline.current, saved, field);
    if (useSaved) {
      form.setFields([
        {
          name: field,
          value: formValues(saved)[field],
          touched: false,
          errors: [],
        },
      ]);
    }
    form.scrollToField(field, {
      focus: true,
      block: "nearest",
      scrollMode: "if-needed",
    });
    const remaining = conflict?.fields.filter((candidate) => candidate !== field);
    if (remaining?.length && conflict) {
      setConflict({ ...conflict, fields: remaining });
    } else {
      setConflict(undefined);
      onConflictChange?.(false);
    }
  }

  function reviewFields() {
    const saved = conflict?.saved;
    if (!saved || !conflict) return [];
    return conflict.fields.map((field) => ({
      field,
      label: labels[field],
      saved: describe(field, saved[field]),
      draft: describe(field, form.getFieldValue(field)),
      onUseSaved: () => resolve(field, saved, true),
      onKeepMine: () => resolve(field, saved, false),
    }));
  }

  return {
    submit,
    conflict,
    reviewFields,
    retryConflict: () => {
      if (conflict && !conflict.loading) void loadConflict(conflict.fields);
    },
  };
}
