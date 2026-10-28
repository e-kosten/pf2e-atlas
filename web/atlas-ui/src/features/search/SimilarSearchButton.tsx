import { Button, Form, Typography } from "antd";
import { useState } from "react";
import {
  buildSearchRequest,
  DEFAULT_SEARCH_STATE,
} from "../../shared/filters/searchState";
import {
  SearchPickerModal,
  type PickerSelection,
} from "../../shared/ui/pickers/SearchPickerModal";
import type { SearchWorkspaceState } from "./useSearchWorkspace";

export function SimilarSearchButton({
  workspace,
}: {
  workspace: SearchWorkspaceState;
}) {
  const [open, setOpen] = useState(false);
  const [candidate, setCandidate] = useState<PickerSelection | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [form] = Form.useForm();
  return (
    <>
      <Button
        onClick={() => {
          setCandidate(null);
          setFeedback(null);
          setOpen(true);
        }}
      >
        {workspace.search.similarSeed ? "Change similar record…" : "Similar to…"}
      </Button>
      <SearchPickerModal
        title="Find similar records"
        confirmLabel="Find similar"
        open={open}
        form={form}
        onCancel={() => setOpen(false)}
        selectedRecordKey={candidate?.record.record_key ?? null}
        onSelectionChange={(selection) => {
          setCandidate(selection);
          setFeedback(null);
        }}
        buildRequest={(query) =>
          buildSearchRequest({ ...DEFAULT_SEARCH_STATE, query }, 1)
        }
        emptyPrompt="Search or choose filters."
        emptyResults="No matching records."
        status={
          feedback ? (
            <Typography.Text role="status">{feedback}</Typography.Text>
          ) : undefined
        }
        onFinish={() => {
          if (!candidate) return;
          const generation =
            workspace.filterDiscoveryContext.expected_artifact_generation;
          if (generation && candidate.artifact_generation !== generation) {
            setFeedback(
              "The reference collection changed. Refresh results before changing this search.",
            );
            return;
          }
          workspace.setSearch({
            ...workspace.search,
            query: "",
            similarSeed: candidate.record.record_key,
            traitReferenceSeed: null,
            mode: "similar_records",
          });
          setOpen(false);
        }}
      >
        {candidate && <Typography.Text>{candidate.record.title}</Typography.Text>}
      </SearchPickerModal>
    </>
  );
}
