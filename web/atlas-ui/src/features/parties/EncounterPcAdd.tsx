import { isUnconfirmedMutation } from "../../api/apiError";
import { Button, Form, Input, InputNumber } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";
import { addEncounterCharacter, getEncounter } from "../../api/atlasApi";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";
import { refreshCharacters } from "./characterQueries";

type Fields = {
  name: string;
  level?: number | null;
  maxHp?: number | null;
  initiative?: number | null;
};

export function EncounterPcAdd({
  encounter,
  onClose,
}: {
  encounter: { encounterKey: string; revision: number };
  onClose: () => void;
}) {
  const [closing, setClosing] = useState(false);
  const [requestId] = useState(() => crypto.randomUUID());
  const [form] = Form.useForm<Fields>();
  const id = useId();
  const client = useQueryClient();
  const paused = useSavedWritesPaused();
  const [revision, setRevision] = useState(encounter.revision);
  const review = useMutation({
    retry: false,
    mutationFn: () => getEncounter(encounter.encounterKey),
    onSuccess: (latest) => {
      setRevision(latest.revision);
      save.reset();
    },
  });
  const save = useMutation({
    retry: false,
    mutationFn: (args: Parameters<typeof addEncounterCharacter>) =>
      addEncounterCharacter(...args),
    onError: () => void refreshCharacters(client),
    onSuccess: async (saved) => {
      await refreshCharacters(client);
      if (!saved.available) return;
      setClosing(true);
    },
  });
  const uncertain = Boolean(save.error && isUnconfirmedMutation(save.error));
  const canReview = Boolean(save.error && !isUnconfirmedMutation(save.error));

  return (
    <AtlasModal
      open={!closing}
      title="Add PC"
      width={416}
      onCancel={() => setClosing(true)}
      confirmDiscard={{
        when: () => form.isFieldsTouched(),
        onDiscard: () => setClosing(true),
      }}
      afterClose={onClose}
      onOk={() => {
        if (uncertain && save.variables) save.mutate(save.variables);
        else form.submit();
      }}
      okText={uncertain ? "Retry to confirm" : "Add to encounter"}
      okButtonProps={{ disabled: paused || canReview || review.isPending }}
      cancelButtonProps={{ disabled: save.isPending }}
      confirmLoading={save.isPending}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
    >
      <Form
        name={id}
        layout="vertical"
        form={form}
        disabled={paused || save.isPending || uncertain || review.isPending}
        onFinish={(values: Fields) =>
          save.mutate([
            encounter.encounterKey,
            {
              request_id: requestId,
              expected_encounter_revision: revision,
              source: {
                kind: "manual",
                name: values.name,
                level: values.level ?? undefined,
                max_hp: values.maxHp ?? undefined,
                initiative: values.initiative ?? undefined,
              },
            },
          ])
        }
      >
        {save.error && (
          <ErrorNotice
            error={save.error}
            title="PC could not be added"
            action={
              canReview ? (
                <Button loading={review.isPending} onClick={() => review.mutate()}>
                  Review latest changes
                </Button>
              ) : undefined
            }
          />
        )}
        {save.data && !save.data.available && (
          <p role="status">
            The participant from this request was removed. Close this form to start a
            new addition.
          </p>
        )}
        {review.error && (
          <ErrorNotice error={review.error} title="Encounter could not refresh" />
        )}
        {review.data && (
          <p role="status">
            Latest encounter reviewed. Your character details are retained below.
          </p>
        )}
        <Form.Item
          name="name"
          label="Name"
          rules={[
            {
              required: true,
              whitespace: true,
              max: 255,
              message: "Enter a character name.",
            },
          ]}
        >
          <Input maxLength={255} />
        </Form.Item>
        <div className="encounter-pc-add__stats">
          <Form.Item
            name="level"
            label="Level"
            rules={[
              {
                type: "integer",
                min: 1,
                max: 20,
                message: "Use a level from 1 to 20.",
              },
            ]}
          >
            <InputNumber min={1} max={20} />
          </Form.Item>
          <Form.Item
            name="maxHp"
            label="Max HP"
            rules={[
              {
                type: "integer",
                min: 0,
                max: Number.MAX_SAFE_INTEGER,
                message: "Enter a nonnegative whole number within the supported range.",
              },
            ]}
          >
            <InputNumber aria-label="Max HP" min={0} />
          </Form.Item>
          <Form.Item
            name="initiative"
            label="Initiative"
            rules={[
              {
                type: "integer",
                min: Number.MIN_SAFE_INTEGER,
                max: Number.MAX_SAFE_INTEGER,
                message: "Enter a whole number within the supported range.",
              },
            ]}
          >
            <InputNumber aria-label="Initiative" />
          </Form.Item>
        </div>
      </Form>
    </AtlasModal>
  );
}
