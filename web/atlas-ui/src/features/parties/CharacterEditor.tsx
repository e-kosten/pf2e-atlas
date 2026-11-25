import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Form, Input, Space } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useLayoutEffect, useRef, useState } from "react";
import { useTabCloseGuard } from "../../app/DesktopTabContext";
import {
  createCharacter,
  getCharacter,
  getParty,
  updateCharacter,
} from "../../api/atlasApi";
import type {
  CreateCharacterRequest,
  PcReferenceView,
  PartyView,
  UpdateCharacterRequest,
} from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { DraftDiscardDialog, useDraftNavigation } from "./CharacterDraftNavigation";
import { refreshCharacters } from "./characterQueries";

type Fields = { name: string; level?: string };

function parseLevel(raw: string | undefined) {
  const text = raw?.trim();
  if (!text) return undefined;
  const level = Number(text);
  return /^\d+$/.test(text) && level >= 1 && level <= 20 ? level : "invalid";
}
type CharacterWrite =
  | { characterKey: string; request: UpdateCharacterRequest }
  | { characterKey?: never; request: CreateCharacterRequest };

export function CharacterEditor({
  character,
  party,
  onClose,
  onSaved,
  inline = false,
}: {
  inline?: boolean;
  character?: PcReferenceView;
  party?: PartyView;
  onClose: () => void;
  onSaved: (key: string) => void;
}) {
  const [form] = Form.useForm<Fields>();
  const id = useId();
  const [requestId] = useState(() => crypto.randomUUID());
  const [closing, setClosing] = useState(false);
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const [characterRevision, setCharacterRevision] = useState(
    character?.character_revision,
  );
  const [partyRevision, setPartyRevision] = useState(party?.revision);
  const nameInput = useRef<React.ComponentRef<typeof Input>>(null);
  const inFlight = useRef<Promise<unknown> | null>(null);
  const committed = useRef(false);
  const initial = { name: character?.name ?? "", level: character?.level?.toString() };
  const [fields, setFields] = useState<Fields>(initial);
  useLayoutEffect(() => {
    if (inline) nameInput.current?.focus({ preventScroll: true });
  }, [inline]);
  const review = useMutation({
    retry: false,
    mutationFn: async () => {
      if (character) return { character: await getCharacter(character.character_key) };
      return { party: party ? await getParty(party.party_key) : undefined };
    },
    onSuccess: (latest) => {
      setCharacterRevision(latest.character?.character_revision);
      setPartyRevision(latest.party?.revision);
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: (write: CharacterWrite) =>
      write.characterKey !== undefined
        ? updateCharacter(write.characterKey, write.request)
        : createCharacter(write.request),
    onError: () => void refreshCharacters(client),
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      committed.current = true;
      setClosing(true);
      onSaved(saved.character_key);
    },
  });
  const baseline = review.data?.character ?? character;
  const hasChanges = (draft: Fields) =>
    (draft.name ?? "").trim() !== (baseline?.name ?? "").trim() ||
    parseLevel(draft.level) !== baseline?.level;
  const dirty = () => !committed.current && hasChanges(form.getFieldsValue());
  const changed = hasChanges(fields);
  const pending = () => (committed.current ? null : inFlight.current);
  const { blocker } = useDraftNavigation(dirty, pending);
  const leaving = blocker.status === "blocked";
  useTabCloseGuard(() =>
    inline ? (pending() ? "pending" : dirty() ? "dirty" : undefined) : undefined,
  );
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const blocked = save.isPending || review.isPending || paused;
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  function write(request: CharacterWrite) {
    if (inFlight.current) return;
    inFlight.current = save
      .mutateAsync(request)
      .catch(() => undefined)
      .finally(() => {
        inFlight.current = null;
      });
  }
  function saveDraft({ name, level }: Fields) {
    if (blocked || canReview) return;
    const parsed = parseLevel(level);
    if (parsed === "invalid") return;
    const fields = { name: name.trim(), level: parsed };
    if (baseline?.name === fields.name && baseline.level === fields.level) return;
    if (character) {
      write({
        characterKey: character.character_key,
        request: {
          ...fields,
          level: parsed ?? null,
          expected_revision: characterRevision!,
        },
      });
    } else {
      write({
        request: {
          ...fields,
          request_id: requestId,
          party: party && {
            party_key: party.party_key,
            expected_revision: partyRevision!,
          },
        },
      });
    }
  }
  function submit() {
    if (uncertain && save.variables) write(save.variables);
    else form.submit();
  }
  const close = () => setClosing(true);
  const content = (
    <Form
      name={id}
      form={form}
      layout="vertical"
      initialValues={initial}
      disabled={blocked || uncertain}
      onValuesChange={(_changed, all) => setFields(all)}
      onFinish={saveDraft}
    >
      {save.error && (
        <ErrorNotice
          title="Character could not be saved"
          error={save.error}
          action={
            canReview ? (
              <Button loading={review.isPending} onClick={() => review.mutate()}>
                Review latest changes
              </Button>
            ) : undefined
          }
        />
      )}
      {review.error && (
        <ErrorNotice title="Current values could not load" error={review.error} />
      )}
      {review.data?.character && (
        <p role="status">
          Latest saved character {review.data.character.name}
          {review.data.character.level !== undefined &&
            ` · Level ${review.data.character.level}`}
          . Your edits are retained below.
        </p>
      )}
      {review.data?.party && (
        <p role="status">
          Latest party {review.data.party.name} · {review.data.party.members.length}{" "}
          characters. Your new character is retained below.
        </p>
      )}
      {save.data && !save.data.available && (
        <p role="status">
          The character from this request was deleted. Close this form to start a new
          character.
        </p>
      )}
      <Form.Item
        label="Name"
        name="name"
        rules={[
          {
            required: true,
            whitespace: true,
            max: 255,
            message: "Enter a character name.",
          },
        ]}
      >
        <Input ref={nameInput} maxLength={255} />
      </Form.Item>
      <Form.Item
        label="Level"
        name="level"
        rules={[
          {
            validator: (_rule, raw?: string) =>
              parseLevel(raw) === "invalid"
                ? Promise.reject(new Error("Use a level from 1 to 20."))
                : Promise.resolve(),
          },
        ]}
      >
        <Input inputMode="numeric" placeholder="—" />
      </Form.Item>
    </Form>
  );
  const [confirmInline, setConfirmInline] = useState(false);
  const discard = (
    <DraftDiscardDialog
      open={confirmInline || leaving}
      title={character ? "Discard character edits?" : "Discard new character?"}
      description="The name and level you entered will be discarded."
      onKeep={() => {
        if (leaving) blocker.reset();
        setConfirmInline(false);
      }}
      onDiscard={() => {
        if (leaving) blocker.proceed();
        else onClose();
      }}
    />
  );
  function cancelInline() {
    if (dirty()) setConfirmInline(true);
    else onClose();
  }
  if (inline)
    return (
      // eslint-disable-next-line jsx-a11y/no-static-element-interactions -- Escape from a focused editor control cancels the inline form.
      <section
        className="character-inline-editor"
        onKeyDown={(event) => {
          if (
            event.key === "Escape" &&
            !blocked &&
            !event.defaultPrevented &&
            event.currentTarget.contains(event.target as Node)
          ) {
            event.stopPropagation();
            cancelInline();
          }
        }}
      >
        <h3>New character</h3>
        {content}
        <Space>
          <Button disabled={blocked} onClick={cancelInline}>
            Cancel
          </Button>
          <Button
            type="primary"
            disabled={blocked || canReview || (!uncertain && !fields.name.trim())}
            loading={save.isPending}
            onClick={submit}
          >
            {uncertain ? "Retry to confirm" : "Add to party"}
          </Button>
        </Space>
        {discard}
      </section>
    );
  return (
    <>
      <AtlasModal
        open={!closing}
        title={character ? `Edit ${character.name}` : "New character"}
        afterClose={onClose}
        onCancel={close}
        confirmDiscard={{ when: dirty, onDiscard: close }}
        closable={!save.isPending}
        keyboard={!save.isPending}
        mask={{ closable: !save.isPending }}
        onOk={submit}
        okText={
          uncertain
            ? "Retry to confirm"
            : character
              ? "Save changes"
              : party
                ? "Add to party"
                : "Add character"
        }
        okButtonProps={{
          disabled:
            blocked ||
            canReview ||
            (!uncertain && (character ? !changed : !fields.name.trim())),
        }}
        cancelButtonProps={{ disabled: save.isPending }}
        confirmLoading={save.isPending}
      >
        {content}
      </AtlasModal>
      {discard}
    </>
  );
}
