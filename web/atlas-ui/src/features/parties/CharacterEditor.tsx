import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Form, Input, InputNumber, Space } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useLayoutEffect, useRef, useState } from "react";
import {
  createCharacter,
  getCharacter,
  getParties,
  updateCharacter,
} from "../../api/atlasApi";
import type { PcReferenceView, PartyView } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { useConfirmDangerAction } from "../../shared/ui/actions/useConfirmDangerAction";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";

type Fields = { name: string; level?: number | null };

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
  const submitted = useRef<{
    values: Fields;
    characterRevision?: number;
    partyRevision?: number;
  } | null>(null);
  const nameInput = useRef<React.ComponentRef<typeof Input>>(null);
  const discard = useConfirmDangerAction();
  useLayoutEffect(() => {
    if (inline) nameInput.current?.focus({ preventScroll: true });
  }, [inline]);
  const review = useMutation({
    retry: false,
    mutationFn: async () => {
      if (character) return { character: await getCharacter(character.character_key) };
      const latestParty = (await getParties()).parties.find(
        (item) => item.party_key === party?.party_key,
      );
      if (party && !latestParty) throw new Error("This party is no longer available.");
      return { party: latestParty };
    },
    onSuccess: (latest) => {
      setCharacterRevision(latest.character?.character_revision);
      setPartyRevision(latest.party?.revision);
      submitted.current = null;
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: (values: Fields) => {
      submitted.current ??= { values, characterRevision, partyRevision };
      const checked = submitted.current;
      return character
        ? updateCharacter(character.character_key, {
            expected_revision: checked.characterRevision!,
            name: checked.values.name,
            level: checked.values.level ?? undefined,
          })
        : createCharacter({
            request_id: requestId,
            name: checked.values.name,
            level: checked.values.level ?? undefined,
            party: party
              ? {
                  party_key: party.party_key,
                  expected_revision: checked.partyRevision!,
                }
              : undefined,
          });
    },
    onError: (error) => {
      if (!isUnconfirmedMutation(error)) submitted.current = null;
      void refreshCharacters(client);
    },
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      setClosing(true);
      onSaved(saved.character_key);
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const blocked = save.isPending || review.isPending || paused;
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));
  const close = () => setClosing(true);
  const content = (
    <Form
      name={id}
      form={form}
      layout="vertical"
      initialValues={{ name: character?.name ?? "", level: character?.level }}
      disabled={blocked || uncertain}
      onFinish={(values) => save.mutate(values)}
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
          { type: "integer", min: 1, max: 20, message: "Use a level from 1 to 20." },
        ]}
      >
        <InputNumber min={1} max={20} placeholder="—" />
      </Form.Item>
    </Form>
  );
  if (inline)
    return (
      <section className="character-inline-editor">
        <h3>New character</h3>
        {content}
        <Space>
          <Button
            disabled={blocked}
            onClick={() => {
              if (form.isFieldsTouched())
                discard.confirm({
                  title: "Discard new character?",
                  content: "The name and level you entered will be discarded.",
                  okText: "Discard",
                  onConfirm: onClose,
                });
              else onClose();
            }}
          >
            Cancel
          </Button>
          <Button
            type="primary"
            disabled={blocked || canReview}
            loading={save.isPending}
            onClick={() => form.submit()}
          >
            {uncertain ? "Retry to confirm" : "Add to party"}
          </Button>
        </Space>
        {discard.dialog}
      </section>
    );
  return (
    <AtlasModal
      open={!closing}
      title={character ? `Edit ${character.name}` : "New character"}
      afterClose={onClose}
      onCancel={close}
      confirmDiscard={{ when: () => form.isFieldsTouched(), onDiscard: close }}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
      onOk={() => form.submit()}
      okText={
        uncertain
          ? "Retry to confirm"
          : character
            ? "Save changes"
            : party
              ? "Add to party"
              : "Add character"
      }
      okButtonProps={{ disabled: blocked || canReview }}
      cancelButtonProps={{ disabled: save.isPending }}
      confirmLoading={save.isPending}
    >
      {content}
    </AtlasModal>
  );
}
