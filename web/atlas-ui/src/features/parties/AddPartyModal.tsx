import { Alert, Button, Checkbox, Form, Modal, Select, Space } from "antd";
import { useRef, useState, type ReactNode } from "react";
import type { AddEncounterPartyRequest, PartyView } from "../../generated/atlas";
import { PartyManager } from "./PartyManager";
import { useParties } from "./useParties";

export function AddPartyModal({
  onClose,
  onAdd,
  disabled,
  recovery,
}: {
  onClose: () => void;
  onAdd: (request: AddEncounterPartyRequest) => Promise<void>;
  disabled: boolean;
  recovery?: ReactNode;
}) {
  const parties = useParties();
  const [party, setParty] = useState<PartyView | null>(null);
  const [members, setMembers] = useState<string[]>([]);
  const [managing, setManaging] = useState(false);
  const [error, setError] = useState<string>();
  const inFlight = useRef(false);
  const submit = async () => {
    if (!party || members.length === 0 || inFlight.current || disabled) return;
    inFlight.current = true;
    setError(undefined);
    try {
      await onAdd({
        party_key: party.party_key,
        expected_revision: party.revision,
        member_keys: members,
      });
      onClose();
    } catch (failure) {
      setError(
        failure instanceof Error ? failure.message : "Party could not be added.",
      );
    } finally {
      inFlight.current = false;
    }
  };
  if (managing) return <PartyManager onClose={() => setManaging(false)} />;
  return (
    <Modal
      open
      title="Add Party"
      okText="Add"
      onOk={() => void submit()}
      onCancel={onClose}
      confirmLoading={disabled}
      okButtonProps={{ disabled: disabled || !party || members.length === 0 }}
      cancelButtonProps={{ disabled }}
      closable={!disabled}
      keyboard={!disabled}
      mask={{ closable: !disabled }}
    >
      {recovery}
      {error && (
        <Alert type="error" title="Party could not be added" description={error} />
      )}
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
      <Form layout="vertical">
        <Form.Item label="Party">
          <Select
            aria-label="Party"
            placeholder="Choose a party"
            loading={parties.isLoading}
            disabled={disabled}
            value={party?.party_key}
            options={(parties.data?.parties ?? []).map((p) => ({
              value: p.party_key,
              label: p.name,
            }))}
            onChange={(key) => {
              const selected =
                parties.data?.parties.find((p) => p.party_key === key) ?? null;
              setParty(selected);
              setMembers(selected?.members.map((m) => m.member_key) ?? []);
              setError(undefined);
            }}
          />
        </Form.Item>
        {party &&
          (party.members.length === 0 ? (
            <p>No PCs in this party.</p>
          ) : (
            <Form.Item label="PCs">
              <Checkbox.Group
                disabled={disabled}
                value={members}
                onChange={(keys) => setMembers(keys as string[])}
              >
                <Space orientation="vertical">
                  {party.members.map((m) => (
                    <Checkbox key={m.member_key} value={m.member_key}>
                      {m.name}
                    </Checkbox>
                  ))}
                </Space>
              </Checkbox.Group>
            </Form.Item>
          ))}
        {!parties.isLoading && !parties.error && parties.data?.parties.length === 0 && (
          <p>No parties yet.</p>
        )}
        <Button
          disabled={disabled}
          onClick={() => {
            setParty(null);
            setMembers([]);
            setManaging(true);
          }}
        >
          Manage Parties
        </Button>
      </Form>
    </Modal>
  );
}
