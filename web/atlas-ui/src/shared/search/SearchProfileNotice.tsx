import { Alert, Button } from "antd";
import { useSearchProfile } from "./SearchProfileContext";

export function SearchProfileNotice() {
  const profile = useSearchProfile();
  if (!profile.error) return null;
  return (
    <Alert
      type="error"
      showIcon
      title={profile.error}
      action={
        profile.canResetLink && (
          <Button onClick={profile.resetLink}>Reset search link</Button>
        )
      }
    />
  );
}
