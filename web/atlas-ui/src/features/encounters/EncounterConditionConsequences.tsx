import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Form, InputNumber, Space } from "antd";
import { getEncounterConditionConsequences } from "../../api/atlasApi";
import type { ResolveDrainedConsequenceRequest } from "../../generated/atlas";
export function EncounterConditionConsequences({
  encounterRef,
  participantKey,
  encounterRevision,
  disabled,
  pending,
  onResolve,
}: {
  encounterRef: string;
  participantKey: string;
  encounterRevision: bigint;
  disabled: boolean;
  pending: boolean;
  onResolve: (
    request: Omit<ResolveDrainedConsequenceRequest, "request_id">,
  ) => Promise<boolean>;
}) {
  const query = useQuery({
    queryKey: ["condition-consequences", encounterRef, encounterRevision.toString()],
    queryFn: () => getEncounterConditionConsequences(encounterRef),
    placeholderData: (previous) => previous,
  });
  if (query.error)
    return (
      <Alert
        type="error"
        title="Could not load HP consequences"
        action={<Button onClick={() => void query.refetch()}>Retry</Button>}
      />
    );
  return (
    <>
      {query.data
        ?.filter((value) => value.participant_key === participantKey)
        .map((value) => (
          <Form
            key={value.consequence_key}
            layout="inline"
            disabled={disabled || pending || query.isFetching}
            onFinish={async (values: { level: number }) => {
              try {
                await onResolve({
                  expected_encounter_revision: encounterRevision,
                  consequence_key: value.consequence_key,
                  level: BigInt(values.level),
                });
              } catch {
                /* Preserve this level until a confirmed refresh removes the consequence. */
              }
            }}
          >
            <Space wrap>
              <span>Drained HP loss</span>
              <Form.Item
                name="level"
                label="Level"
                rules={[
                  {
                    validator: (_rule, level) =>
                      Number.isSafeInteger(level)
                        ? Promise.resolve()
                        : Promise.reject(new Error("Enter a whole number.")),
                  },
                ]}
              >
                <InputNumber aria-label="Level for Drained HP loss" />
              </Form.Item>
              <Button htmlType="submit" loading={pending}>
                Apply
              </Button>
            </Space>
          </Form>
        ))}
    </>
  );
}
