import { ErrorNotice } from "../feedback/ErrorNotice";
import { Button, Space, Typography } from "antd";
import { useSearchProfile } from "./SearchProfileContext";

export function SearchProfileNotice({
  similar = false,
  showKeyword = false,
  rerunSemantic,
}: { similar?: boolean; showKeyword?: boolean; rerunSemantic?: () => void } = {}) {
  const profile = useSearchProfile();
  const defaultModel = profile.profiles.find((model) => model.id === profile.defaultId);
  const canSwitch =
    profile.keywordSearch &&
    (similar ? defaultModel?.stored_vectors : defaultModel?.usable);
  const switchButton = canSwitch && (
    <Button type="link" onClick={profile.resetLink}>
      Use semantic search
    </Button>
  );
  const error =
    profile.error ??
    (similar && profile.keywordSearch
      ? canSwitch
        ? "Switch to semantic search to find similar records."
        : "Similar records need a semantic index."
      : null);
  if (!error)
    return showKeyword ? (
      <>
        {profile.keywordSearch && (
          <Space className="search-profile-status">
            <Typography.Text type="secondary" role="status">
              {rerunSemantic ? "Keyword results" : "Keyword search"}
            </Typography.Text>
            {rerunSemantic && (
              <Button type="link" onClick={rerunSemantic}>
                Rerun this search
              </Button>
            )}
          </Space>
        )}
      </>
    ) : null;
  return (
    <ErrorNotice
      error={error}
      title={error}
      action={
        (profile.canResetLink || profile.useKeywordSearch || canSwitch) && (
          <Space wrap>
            {profile.canResetLink && profile.defaultId !== null && (
              <Button onClick={profile.resetLink}>Reset search link</Button>
            )}
            {profile.useKeywordSearch && (
              <Button onClick={profile.useKeywordSearch}>Use keyword search</Button>
            )}
            {switchButton}
          </Space>
        )
      }
    />
  );
}
