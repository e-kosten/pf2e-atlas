import { Alert, Button, Spin } from "antd";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { verifyBackendCompatibility } from "../api/atlasApi";
import { API_CONTRACT, AtlasCompatibilityError } from "../api/compatibility";

export function BackendCompatibilityGate({ children }: { children: ReactNode }) {
  const check = useQuery({
    queryKey: ["backendCompatibility", API_CONTRACT],
    queryFn: verifyBackendCompatibility,
    retry: false,
    // Each API request also advertises this contract. Don't unmount editors and
    // discard drafts when a background focus/reconnect check hits an outage.
    staleTime: Infinity,
  });
  if (check.isError) {
    return (
      <main className="backend-status">
        <Alert
          type="error"
          showIcon
          title={
            check.error instanceof AtlasCompatibilityError
              ? "Atlas versions don’t match"
              : "Unable to connect to Atlas"
          }
          description={check.error.message}
          action={
            <Button loading={check.isFetching} onClick={() => void check.refetch()}>
              Retry
            </Button>
          }
        />
      </main>
    );
  }
  if (!check.isSuccess) {
    return (
      <main className="backend-status" aria-label="Connecting to Atlas">
        <Spin delay={250} />
      </main>
    );
  }
  return children;
}
