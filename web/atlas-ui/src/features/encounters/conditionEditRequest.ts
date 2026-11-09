export type ConditionEditRequest = {
  participantKey: string;
  conditionId: number;
  token: string;
};
export type EditConditionSource = (participantKey: string, conditionId: number) => void;
