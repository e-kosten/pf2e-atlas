export type ConditionEditRequest = {
  participantKey: string;
  conditionId: bigint;
  token: string;
};
export type EditConditionSource = (participantKey: string, conditionId: bigint) => void;
