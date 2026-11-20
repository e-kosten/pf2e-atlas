import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button, Checkbox, Form, Input, InputNumber } from "antd";
import type { InputRef } from "antd";
import { useId, useRef, useState, type ReactNode } from "react";
import { AtlasApiError, getParties, updateParty } from "../../api/atlasApi";
import type { PartyView, UpdatePartyRequest } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";

function partyMemberships(party: PartyView) {
  return party.members.map(({ character_key, character_revision }) => ({
    character_key,
    expected_character_revision: character_revision,
  }));
}

function usePartyEdit(
  initialParty: PartyView,
  onReviewed: (latest: PartyView, previous: PartyView) => void,
) {
  const [party, setParty] = useState(initialParty);
  const [closing, setClosing] = useState(false);
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  const save = useMutation({
    retry: false,
    mutationFn: (request: UpdatePartyRequest) => updateParty(party.party_key, request),
    onSuccess: async () => {
      await refreshCharacters(client);
      setClosing(true);
    },
  });
  const review = useMutation({
    retry: false,
    mutationFn: async () => {
      const index = await getParties();
      const latest = index.parties.find((item) => item.party_key === party.party_key);
      if (!latest) throw new Error("This party was deleted. You can close this form.");
      return latest;
    },
    onSuccess: (latest) => {
      onReviewed(latest, party);
      setParty(latest);
      save.reset();
    },
  });
  return {
    party,
    save,
    review,
    closing,
    close: () => setClosing(true),
    busy: paused || save.isPending || review.isPending,
    stale: save.error instanceof AtlasApiError && save.error.status === 409,
  };
}

function PartyEditDialog({
  edit,
  title,
  action,
  dirty,
  canSave,
  onSubmit,
  onClose,
  focusInput,
  children,
}: {
  edit: ReturnType<typeof usePartyEdit>;
  title: string;
  action: string;
  dirty: boolean;
  canSave: boolean;
  onSubmit: () => void;
  onClose: () => void;
  focusInput: () => void;
  children: ReactNode;
}) {
  return (
    <AtlasModal
      open={!edit.closing}
      afterClose={onClose}
      afterOpenChange={(shown) => {
        if (shown) focusInput();
      }}
      title={title}
      okText={action}
      onOk={onSubmit}
      onCancel={edit.close}
      confirmDiscard={{ when: () => dirty, onDiscard: edit.close }}
      confirmLoading={edit.save.isPending}
      closable={!edit.save.isPending}
      keyboard={!edit.save.isPending}
      mask={{ closable: !edit.save.isPending }}
      okButtonProps={{ disabled: !canSave }}
      cancelButtonProps={{ disabled: edit.save.isPending }}
    >
      {edit.save.error && (
        <ErrorNotice
          title="Party could not be saved"
          error={edit.save.error}
          action={
            edit.stale && (
              <Button
                loading={edit.review.isPending}
                onClick={() => edit.review.mutate()}
              >
                Review latest changes
              </Button>
            )
          }
        />
      )}
      {edit.review.error && (
        <ErrorNotice
          title="Latest party changes could not load"
          error={edit.review.error}
        />
      )}
      {edit.review.data && !edit.stale && (
        <p className="party-edit-review" role="status">
          Latest changes reviewed. Your draft is retained below.
        </p>
      )}
      {children}
    </AtlasModal>
  );
}

export function RenameParty({
  party,
  onClose,
}: {
  party: PartyView;
  onClose: () => void;
}) {
  const [form] = Form.useForm<{ name: string }>();
  const id = useId();
  const input = useRef<InputRef>(null);
  const edit = usePartyEdit(party, (latest, previous) => {
    if (String(form.getFieldValue("name") ?? "").trim() === previous.name.trim())
      form.setFieldValue("name", latest.name);
  });
  const name = Form.useWatch("name", form) ?? party.name;
  const dirty = name.trim() !== edit.party.name.trim();
  const valid = name.trim().length > 0 && [...name.trim()].length <= 255;
  const canSave = dirty && valid && !edit.busy && !edit.stale;
  return (
    <PartyEditDialog
      edit={edit}
      title="Rename party"
      action="Rename"
      dirty={dirty}
      canSave={canSave}
      onSubmit={() => form.submit()}
      onClose={onClose}
      focusInput={() => input.current?.focus({ cursor: "all", preventScroll: true })}
    >
      {edit.review.data && (
        <p className="party-edit-saved">Saved name: {edit.party.name}</p>
      )}
      <Form
        form={form}
        name={id}
        layout="vertical"
        initialValues={{ name: party.name }}
        disabled={edit.busy}
        onFinish={({ name: nextName }) => {
          if (canSave)
            edit.save.mutate({
              name: nextName,
              expected_revision: edit.party.revision,
              members: partyMemberships(edit.party),
              level_updates: [],
            });
        }}
      >
        <Form.Item
          name="name"
          label="Party name"
          rules={[{ required: true, whitespace: true, message: "Enter a party name." }]}
        >
          <Input ref={input} maxLength={255} />
        </Form.Item>
      </Form>
    </PartyEditDialog>
  );
}

export function SetPartyLevels({
  party,
  onClose,
}: {
  party: PartyView;
  onClose: () => void;
}) {
  const [selected, setSelected] = useState(
    party.members.map((member) => member.character_key),
  );
  const [target, setTarget] = useState<{ value: number | null; raw: string }>({
    value: null,
    raw: "",
  });
  const input = useRef<React.ComponentRef<typeof InputNumber>>(null);
  const id = useId();
  const edit = usePartyEdit(party, (latest) => {
    setSelected((keys) =>
      keys.filter((key) =>
        latest.members.some((member) => member.character_key === key),
      ),
    );
  });
  const raw = target.raw.trim();
  const level = /^\d+$/.test(raw) ? Number(raw) : undefined;
  const valid =
    level !== undefined && Number.isInteger(level) && level >= 1 && level <= 20;
  const changes = valid
    ? edit.party.members.filter(
        (member) => selected.includes(member.character_key) && member.level !== level,
      )
    : [];
  const dirty = raw !== "" && (!valid || changes.length > 0);
  const canSave = valid && changes.length > 0 && !edit.busy && !edit.stale;
  const action =
    valid && changes.length > 0
      ? `Set ${changes.length} ${changes.length === 1 ? "character" : "characters"} to level ${level}`
      : "Set character levels";
  const submit = () => {
    if (canSave)
      edit.save.mutate({
        name: edit.party.name,
        expected_revision: edit.party.revision,
        members: partyMemberships(edit.party),
        level_updates: changes.map((member) => ({
          character_key: member.character_key,
          expected_character_revision: member.character_revision,
          level,
        })),
      });
  };
  return (
    <PartyEditDialog
      edit={edit}
      title="Set character levels"
      action={action}
      dirty={dirty}
      canSave={canSave}
      onSubmit={submit}
      onClose={onClose}
      focusInput={() => input.current?.focus({ cursor: "all", preventScroll: true })}
    >
      <form
        id={id}
        onSubmit={(event) => {
          event.preventDefault();
          submit();
        }}
      >
        <div className="party-level-target">
          <label htmlFor={`${id}-level`}>New level</label>
          <InputNumber
            id={`${id}-level`}
            ref={input}
            min={1}
            max={20}
            value={target.value}
            placeholder="1–20"
            changeOnBlur={false}
            disabled={edit.busy}
            status={raw !== "" && !valid ? "error" : undefined}
            aria-invalid={raw !== "" && !valid}
            aria-describedby={raw !== "" && !valid ? `${id}-error` : undefined}
            onChange={(value) =>
              setTarget({ value, raw: value == null ? "" : String(value) })
            }
            onInput={(text) => setTarget((current) => ({ ...current, raw: text }))}
            onKeyDownCapture={(event) => {
              // InputNumber otherwise normalizes partial text before the form sees Enter.
              if (event.key === "Enter" && !valid) {
                event.preventDefault();
                event.stopPropagation();
              }
            }}
          />
          {raw !== "" && !valid && (
            <span id={`${id}-error`} role="alert">
              Use a whole number from 1 to 20.
            </span>
          )}
        </div>
        <fieldset className="party-level-roster" disabled={edit.busy}>
          <legend>Characters</legend>
          {edit.party.members.map((member) => (
            <div className="party-level-row" key={member.character_key}>
              <Checkbox
                disabled={edit.busy}
                checked={selected.includes(member.character_key)}
                onChange={(event) =>
                  setSelected((keys) =>
                    event.target.checked
                      ? [...keys, member.character_key]
                      : keys.filter((key) => key !== member.character_key),
                  )
                }
              >
                {member.name}
              </Checkbox>
              <span className="party-level-current">
                {member.level === undefined ? "No level" : `Level ${member.level}`}
              </span>
            </div>
          ))}
        </fieldset>
        {valid && changes.length === 0 && (
          <p className="party-edit-hint" role="status">
            {selected.length === 0
              ? "Select a character to update."
              : "The selected characters already have this level."}
          </p>
        )}
      </form>
    </PartyEditDialog>
  );
}
