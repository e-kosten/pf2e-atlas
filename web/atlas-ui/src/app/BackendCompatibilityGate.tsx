import { Alert, Button, Spin } from "antd";
import { useQuery } from "@tanstack/react-query";
import { useEffect, type ReactNode } from "react";
import { verifyBackendCompatibility } from "../api/atlasApi";
import { API_CONTRACT, AtlasCompatibilityError } from "../api/compatibility";
import { reloadFrontend } from "../shared/browser/reloadFrontend";
import { errorExplanation } from "../shared/feedback/errorMessages";
import {
  clearStartupReloadAttempt,
  reloadMismatchedFrontendOnce,
} from "../shared/browser/startupReload";

export function BackendCompatibilityGate({ children }: { children: ReactNode }) {
  const development = import.meta.env.DEV;
  const installed = import.meta.env.PROD;
  const check = useQuery({
    queryKey: ["backendCompatibility", API_CONTRACT],
    queryFn: verifyBackendCompatibility,
    retry: false,
    // Each API request also advertises this contract. Don't unmount editors and
    // discard drafts when a background focus/reconnect check hits an outage.
    staleTime: Infinity,
  });
  const mismatch = check.error instanceof AtlasCompatibilityError;
  useEffect(() => {
    if (check.isSuccess) clearStartupReloadAttempt();
    else if (mismatch && installed) reloadMismatchedFrontendOnce();
  }, [check.isSuccess, mismatch, installed]);
  if (check.isError) {
    return (
      <main className="backend-status">
        <Alert
          type="error"
          showIcon
          title={mismatch ? "Atlas versions don’t match" : "Unable to connect to Atlas"}
          description={
            mismatch && development
              ? "Rebuild and restart the backend from this checkout, then check ATLAS_API_PROXY."
              : errorExplanation(check.error)
          }
          action={
            <Button
              loading={check.isFetching}
              onClick={
                mismatch && installed ? reloadFrontend : () => void check.refetch()
              }
            >
              {mismatch && installed ? "Reload page" : "Retry"}
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
