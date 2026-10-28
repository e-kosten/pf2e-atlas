import { ErrorNotice } from "../feedback/ErrorNotice";
import { Button } from "antd";
import { useSearchProfile } from "./SearchProfileContext";

export function SearchProfileNotice() {
  const profile = useSearchProfile();
  if (!profile.error) return null;
  return (
    <ErrorNotice
      error={profile.error}
      title={profile.error}
      action={
        profile.canResetLink && (
          <Button onClick={profile.resetLink}>Reset search link</Button>
        )
      }
    />
  );
}
