import { AtlasModal as Modal } from "../../shared/ui/overlays/AtlasModal";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Form,
  Input,
  InputNumber,
  Segmented,
  Space,
  Table,
  Tooltip,
} from "antd";
import { Trash2 } from "lucide-react";
import { useId, useRef, useState } from "react";
import { createParty, deleteParty, updateParty } from "../../api/atlasApi";
import type {
  CreatePartyRequest,
  PartyIndexView,
  PartyView,
} from "../../generated/atlas";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { useParties } from "./useParties";

export function PartyManager({
  onClose,
  initialCreate = false,
}: {
  onClose: () => void;
  initialCreate?: boolean;
}) {
  const parties = useParties();
  const client = useQueryClient();
  const [closing, setClosing] = useState(false);
  const [editor, setEditor] = useState<PartyView | "new" | null>(
    initialCreate ? "new" : null,
  );
  const remove = useMutation({
    retry: false,
    mutationFn: (party: PartyView) => deleteParty(party.party_key, party.revision),
    onSuccess: (receipt) => {
      client.setQueryData<PartyIndexView>(["parties"], (previous) => ({
        parties: (previous?.parties ?? []).filter(
          (p) => p.party_key !== receipt.party_key,
        ),
      }));
      void client.invalidateQueries({ queryKey: ["parties"] });
    },
  });
  if (initialCreate) return <PartyEditor onClose={onClose} />;
  return (
    <>
      <Modal
        open={!closing}
        afterClose={onClose}
        title="Parties"
        onCancel={() => setClosing(true)}
        closable={!remove.isPending && !editor}
        keyboard={!remove.isPending && !editor}
        mask={{ closable: !remove.isPending && !editor }}
        footer={
          <Space>
            <Button disabled={remove.isPending} onClick={() => setClosing(true)}>
              Done
            </Button>
            <Button
              type="primary"
              disabled={remove.isPending}
              onClick={(event) => {
                event.currentTarget.focus();
                setEditor("new");
              }}
            >
              New Party
            </Button>
          </Space>
        }
      >
        {parties.error && (
          <ErrorNotice
            error={parties.error}
            title="Parties could not load"
            action={
              <Button
                loading={parties.isFetching}
                onClick={() => void parties.refetch()}
              >
                Retry
              </Button>
            }
          />
        )}
        {remove.error && (
          <ErrorNotice error={remove.error} title="Party could not be deleted" />
        )}
        <Table<PartyView>
          rowKey="party_key"
          size="small"
          tableLayout="fixed"
          pagination={false}
          loading={parties.isLoading}
          dataSource={parties.data?.parties ?? []}
          locale={{
            emptyText: parties.error ? "Parties unavailable" : "No parties yet.",
          }}
          columns={[
            {
              title: "Party",
              dataIndex: "name",
              render: (_value, party) => (
                <Button
                  type="link"
                  block
                  style={{
                    height: "auto",
                    paddingInline: 0,
                    whiteSpace: "normal",
                    overflowWrap: "anywhere",
                    textAlign: "start",
                    justifyContent: "start",
                  }}
                  disabled={remove.isPending}
                  onClick={(event) => {
                    event.currentTarget.focus();
                    setEditor(party);
                  }}
                >
                  {party.name}
                </Button>
              ),
            },
            {
              title: "PCs",
              width: 48,
              render: (_value, party) => party.members.length,
            },
            {
              title: "",
              width: 84,
              render: (_value, party) => (
                <DangerActionButton
                  disabled={remove.isPending}
                  aria-label={`Delete ${party.name}`}
                  confirmTitle={`Delete ${party.name}?`}
                  confirmContent="PCs already added to encounters will remain."
                  confirmOkText="Delete"
                  onConfirm={async () => {
                    await remove.mutateAsync(party);
                  }}
                >
                  Delete
                </DangerActionButton>
              ),
            },
          ]}
        />
      </Modal>
      {editor && (
        <PartyEditor
          party={editor === "new" ? undefined : editor}
          onClose={() => setEditor(null)}
        />
      )}
    </>
  );
}

type PartyLevelMode = "shared" | "individual";
type PartyEditorValues = CreatePartyRequest & { sharedLevel?: number | null };

function PartyEditor({ party, onClose }: { party?: PartyView; onClose: () => void }) {
  const [form] = Form.useForm<PartyEditorValues>();
  const initialMembers = party?.members ?? [];
  const uniformLevels = initialMembers.every(
    (member) => member.level === initialMembers[0]?.level,
  );
  const [levelMode, setLevelMode] = useState<PartyLevelMode>(
    !party || (uniformLevels && initialMembers[0]?.level !== undefined)
      ? "shared"
      : "individual",
  );
  const memberCount = (Form.useWatch("members", form) ?? initialMembers).length;
  const individualDraftReady = useRef(initialMembers.length > 0);
  const [closing, setClosing] = useState(false);
  const id = useId();
  const inFlight = useRef(false);
  const client = useQueryClient();
  const save = useMutation({
    retry: false,
    mutationFn: (values: CreatePartyRequest) =>
      party
        ? updateParty(party.party_key, { ...values, expected_revision: party.revision })
        : createParty(values),
    onSuccess: (saved) => {
      client.setQueryData<PartyIndexView>(["parties"], (previous) => ({
        parties: [
          ...(previous?.parties ?? []).filter((p) => p.party_key !== saved.party_key),
          saved,
        ],
      }));
      void client.invalidateQueries({ queryKey: ["parties"] });
      setClosing(true);
    },
  });
  const changeLevelMode = (next: PartyLevelMode) => {
    if (next === levelMode) return;
    const members = form.getFieldValue("members") as CreatePartyRequest["members"];
    if (next === "individual" && !individualDraftReady.current) {
      const level = form.getFieldValue("sharedLevel") as number | null | undefined;
      members.forEach((_member, index) =>
        form.setFieldValue(["members", index, "level"], level ?? undefined),
      );
      individualDraftReady.current = true;
    } else if (
      next === "shared" &&
      members.length > 0 &&
      members.every((member) => member.level === members[0].level)
    ) {
      form.setFieldValue("sharedLevel", members[0].level);
    }
    setLevelMode(next);
  };
  const submit = async (values: PartyEditorValues) => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      await save.mutateAsync({
        name: values.name,
        members: values.members.map((member) => ({
          ...member,
          level:
            (levelMode === "shared" ? values.sharedLevel : member.level) ?? undefined,
        })),
      });
    } catch {
      /* Keep the draft and show the mutation error. */
    } finally {
      inFlight.current = false;
    }
  };
  return (
    <Modal
      open={!closing}
      afterClose={onClose}
      title={party ? "Edit Party" : "New Party"}
      okText="Save"
      onOk={() => form.submit()}
      onCancel={() => setClosing(true)}
      confirmLoading={save.isPending}
      cancelButtonProps={{ disabled: save.isPending }}
      closable={!save.isPending}
      keyboard={!save.isPending}
      mask={{ closable: !save.isPending }}
    >
      <Form
        name={id}
        layout="vertical"
        form={form}
        disabled={save.isPending}
        initialValues={{
          ...(party ?? { name: "", members: [] }),
          sharedLevel: uniformLevels ? initialMembers[0]?.level : undefined,
        }}
        onFinish={(values) => void submit(values)}
      >
        {save.error && (
          <ErrorNotice error={save.error} title="Party could not be saved" />
        )}
        <Form.Item
          label="Party name"
          name="name"
          rules={[
            {
              required: true,
              whitespace: true,
              max: 255,
              message: "Enter a party name.",
            },
          ]}
        >
          <Input maxLength={255} />
        </Form.Item>
        <div className="party-editor__level-mode">
          <Form.Item label="PC levels">
            <Segmented<PartyLevelMode>
              aria-label="PC levels"
              name={`${id}-level-mode`}
              value={levelMode}
              onChange={changeLevelMode}
              options={[
                { label: "Same level", value: "shared" },
                { label: "Individual levels", value: "individual" },
              ]}
            />
          </Form.Item>
          <Form.Item
            name="sharedLevel"
            label="Party level"
            hidden={levelMode !== "shared"}
            rules={
              levelMode === "shared"
                ? [
                    {
                      required: memberCount > 0,
                      type: "integer",
                      min: 1,
                      max: 20,
                      message: "Use 1–20.",
                    },
                  ]
                : []
            }
          >
            <InputNumber aria-label="Party level" min={1} max={20} placeholder="—" />
          </Form.Item>
        </div>
        <Form.List name="members">
          {(fields, { add, remove }) => (
            <div className="party-editor__members">
              {fields.length > 0 && (
                <div
                  className={`party-editor__row party-editor__row--${levelMode} party-editor__headings`}
                  aria-hidden="true"
                >
                  <span>PC name</span>
                  {levelMode === "individual" && <span>Level</span>}
                  <span />
                </div>
              )}
              {fields.map(({ key, name, ...rest }) => (
                <div
                  key={key}
                  className={`party-editor__row party-editor__row--${levelMode}`}
                >
                  <Form.Item {...rest} name={[name, "member_key"]} hidden>
                    <Input />
                  </Form.Item>
                  <Form.Item
                    {...rest}
                    className="party-editor__field"
                    name={[name, "name"]}
                    rules={[
                      {
                        required: true,
                        whitespace: true,
                        max: 255,
                        message: "Enter a PC name.",
                      },
                    ]}
                  >
                    <Input maxLength={255} aria-label={`PC ${name + 1} name`} />
                  </Form.Item>
                  <Form.Item
                    {...rest}
                    className="party-editor__field"
                    name={[name, "level"]}
                    hidden={levelMode !== "individual"}
                    rules={
                      levelMode === "individual"
                        ? [{ type: "integer", min: 1, max: 20, message: "Use 1–20." }]
                        : []
                    }
                  >
                    <InputNumber
                      className="party-editor__level"
                      aria-label={`PC ${name + 1} level`}
                      min={1}
                      max={20}
                      placeholder="—"
                    />
                  </Form.Item>
                  <Tooltip title={`Remove PC ${name + 1}`} trigger={["hover", "focus"]}>
                    <Button
                      type="text"
                      aria-label={`Remove PC ${name + 1}`}
                      onClick={() => remove(name)}
                      icon={<Trash2 size={16} aria-hidden="true" />}
                    />
                  </Tooltip>
                </div>
              ))}
              <Button
                className="party-editor__add"
                disabled={save.isPending || fields.length >= 50}
                onClick={() =>
                  add({
                    name: "",
                    ...(levelMode === "shared" && individualDraftReady.current
                      ? { level: form.getFieldValue("sharedLevel") ?? undefined }
                      : {}),
                  })
                }
              >
                Add PC
              </Button>
            </div>
          )}
        </Form.List>
      </Form>
    </Modal>
  );
}
