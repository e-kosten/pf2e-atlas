import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Form, Input, Modal, Space, Table } from "antd";
import { useId, useRef, useState } from "react";
import { createParty, deleteParty, updateParty } from "../../api/atlasApi";
import type {
  CreatePartyRequest,
  PartyIndexView,
  PartyView,
} from "../../generated/atlas";
import { DangerActionButton } from "../../shared/ui/actions/DangerActionButton";
import { useParties } from "./useParties";

export function PartyManager({ onClose }: { onClose: () => void }) {
  const parties = useParties();
  const client = useQueryClient();
  const [editor, setEditor] = useState<PartyView | "new" | null>(null);
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
  if (editor)
    return (
      <PartyEditor
        party={editor === "new" ? undefined : editor}
        onClose={() => setEditor(null)}
      />
    );
  return (
    <Modal
      open
      title="Parties"
      onCancel={onClose}
      closable={!remove.isPending}
      keyboard={!remove.isPending}
      mask={{ closable: !remove.isPending }}
      footer={
        <Space>
          <Button disabled={remove.isPending} onClick={onClose}>
            Done
          </Button>
          <Button
            type="primary"
            disabled={remove.isPending}
            onClick={() => setEditor("new")}
          >
            New Party
          </Button>
        </Space>
      }
    >
      {parties.error && (
        <Alert
          type="error"
          title="Parties could not load"
          description={parties.error.message}
          action={
            <Button loading={parties.isFetching} onClick={() => void parties.refetch()}>
              Retry
            </Button>
          }
        />
      )}
      {remove.error && (
        <Alert
          type="error"
          title="Party could not be deleted"
          description={remove.error.message}
        />
      )}
      <Table<PartyView>
        rowKey="party_key"
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
                disabled={remove.isPending}
                onClick={() => setEditor(party)}
              >
                {party.name}
              </Button>
            ),
          },
          { title: "PCs", render: (_value, party) => party.members.length },
          {
            title: "",
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
  );
}

function PartyEditor({ party, onClose }: { party?: PartyView; onClose: () => void }) {
  const [form] = Form.useForm<CreatePartyRequest>();
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
      onClose();
    },
  });
  const submit = async (values: CreatePartyRequest) => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      await save.mutateAsync(values);
    } catch {
      /* Keep the draft and show the mutation error. */
    } finally {
      inFlight.current = false;
    }
  };
  return (
    <Modal
      open
      title={party ? "Edit Party" : "New Party"}
      okText="Save"
      onOk={() => form.submit()}
      onCancel={onClose}
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
        initialValues={party ?? { name: "", members: [] }}
        onFinish={(values) => void submit(values)}
      >
        {save.error && (
          <Alert
            type="error"
            title="Party could not be saved"
            description={save.error.message}
          />
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
        <Form.List name="members">
          {(fields, { add, remove }) => (
            <>
              {fields.map(({ key, name, ...rest }) => (
                <Space key={key} align="start" style={{ display: "flex" }}>
                  <Form.Item {...rest} name={[name, "member_key"]} hidden>
                    <Input />
                  </Form.Item>
                  <Form.Item
                    {...rest}
                    label={`PC ${name + 1}`}
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
                    <Input maxLength={255} />
                  </Form.Item>
                  <Button
                    style={{ marginTop: 30 }}
                    aria-label={`Remove PC ${name + 1}`}
                    onClick={() => remove(name)}
                  >
                    Remove
                  </Button>
                </Space>
              ))}
              <Button
                disabled={save.isPending || fields.length >= 50}
                onClick={() => add({ name: "" })}
              >
                Add PC
              </Button>
            </>
          )}
        </Form.List>
      </Form>
    </Modal>
  );
}
