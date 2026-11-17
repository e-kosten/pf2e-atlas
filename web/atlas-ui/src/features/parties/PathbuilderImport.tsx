import { App, Button, Input, List, Space, Typography } from "antd";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";
import { previewPathbuilderPc, savePathbuilderPc } from "../../api/atlasApi";
import type { PartyMemberView, PartyView } from "../../generated/atlas";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useSavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { AtlasModal } from "../../shared/ui/overlays/AtlasModal";

export function PathbuilderImport({
  party,
  member,
  onClose,
}: {
  party: PartyView;
  member?: PartyMemberView;
  onClose: () => void;
}) {
  const textareaId = useId();
  const [text, setText] = useState("");
  const [closing, setClosing] = useState(false);
  const paused = useSavedWritesPaused();
  const client = useQueryClient();
  const { message } = App.useApp();
  const preview = useMutation({
    mutationFn: async (json: string) => ({
      view: await previewPathbuilderPc(json),
      partyRevision: party.revision,
    }),
    retry: false,
  });
  const save = useMutation({
    retry: false,
    mutationFn: (reviewed: NonNullable<typeof preview.data>) =>
      savePathbuilderPc(party.party_key, {
        json: text,
        member_key: member?.member_key,
        expected_party_revision: reviewed.partyRevision,
        expected_artifact_generation: reviewed.view.artifact_generation,
      }),
    onError: () => {
      void client.invalidateQueries({ queryKey: ["parties"] });
    },
    onSuccess: async (saved) => {
      await Promise.all([
        client.invalidateQueries({ queryKey: ["parties"] }),
        client.invalidateQueries({ queryKey: ["pc-reference"] }),
      ]);
      void message.success(
        member ? `${saved.name} refreshed` : `${saved.name} added to the party`,
      );
      setClosing(true);
    },
  });
  const busy = preview.isPending || save.isPending;
  const draft = preview.data?.view;
  return (
    <AtlasModal
      open={!closing}
      title={member ? `Refresh ${member.name}` : "Import from Pathbuilder"}
      onCancel={() => setClosing(true)}
      afterClose={onClose}
      closable={!busy}
      keyboard={!busy}
      mask={{ closable: false }}
      footer={
        <Space wrap>
          <Button onClick={() => setClosing(true)} disabled={busy}>
            Cancel
          </Button>
          <Button
            type={draft ? "default" : "primary"}
            disabled={!text.trim() || save.isPending}
            loading={preview.isPending}
            onClick={() => {
              save.reset();
              preview.mutate(text);
            }}
          >
            Preview import
          </Button>
          {draft && (
            <Button
              type="primary"
              loading={save.isPending}
              disabled={paused || busy}
              onClick={() => {
                if (preview.data) save.mutate(preview.data);
              }}
            >
              {member ? "Refresh character" : "Add to party"}
            </Button>
          )}
        </Space>
      }
    >
      <Space orientation="vertical" size="middle" style={{ display: "flex" }}>
        <div>
          <label htmlFor={textareaId}>
            <Typography.Text strong>Pathbuilder JSON</Typography.Text>
          </label>
          <Input.TextArea
            id={textareaId}
            value={text}
            rows={5}
            autoComplete="off"
            spellCheck={false}
            disabled={busy}
            onChange={(event) => {
              setText(event.target.value);
              preview.reset();
              save.reset();
            }}
          />
        </div>
        {preview.error && (
          <ErrorNotice error={preview.error} title="Could not preview character" />
        )}
        {save.error && (
          <ErrorNotice error={save.error} title="Could not save character" />
        )}
        {draft && (
          <section aria-label="Import preview">
            <Typography.Title level={4} style={{ marginBlockStart: 0 }}>
              {draft.name} · Level {draft.level}
            </Typography.Title>
            {member && (member.name !== draft.name || member.level !== draft.level) && (
              <p>
                Current party entry: {member.name}
                {member.level && ` · Level ${member.level}`}
              </p>
            )}
            <div style={{ maxBlockSize: "min(30vh, 18rem)", overflow: "auto" }}>
              <List
                size="small"
                dataSource={draft.abilities}
                locale={{ emptyText: "No abilities in this export." }}
                renderItem={(ability) => (
                  <List.Item key={ability.ability_key}>
                    <Space wrap>
                      <Typography.Text>
                        {ability.label}
                        {ability.choice && ` (${ability.choice})`}
                      </Typography.Text>
                      <Typography.Text type="secondary">
                        {ability.record_title ??
                          (ability.ambiguous ? "Multiple matches" : "Search needed")}
                      </Typography.Text>
                    </Space>
                  </List.Item>
                )}
              />
            </div>
          </section>
        )}
      </Space>
    </AtlasModal>
  );
}
