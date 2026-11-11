import { Alert, Button } from "antd";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { GlobalFeedback } from "../feedback/GlobalFeedback";

type RerunOwner = { id: symbol; rerun?: () => void };
const RegisterRerun = createContext<((owner: RerunOwner) => () => void) | null>(null);

/** The foremost open picker owns the notice action, including when it has no results. */
export function useSemanticRerunOwner(open: boolean, rerun?: () => void) {
  const register = useContext(RegisterRerun);
  const callback = useRef(rerun);
  useLayoutEffect(() => {
    callback.current = rerun;
  }, [rerun]);
  const canRerun = Boolean(rerun);
  useEffect(() => {
    if (open && register) {
      return register({
        id: Symbol(),
        rerun: canRerun ? () => callback.current?.() : undefined,
      });
    }
  }, [open, register, canRerun]);
}

export function SemanticReadyNotice({
  keywordOnly,
  ready,
  rerun,
  children,
}: {
  keywordOnly: boolean;
  ready: boolean;
  rerun?: () => void;
  children: ReactNode;
}) {
  const [owners, setOwners] = useState<RerunOwner[]>([]);
  const [phase, setPhase] = useState(keywordOnly ? "waiting" : "initial");
  const [focused, setFocused] = useState(false);
  if (phase === "initial" && keywordOnly) setPhase("waiting");
  if (phase === "waiting" && ready) setPhase("showing");
  const register = useCallback((owner: RerunOwner) => {
    setOwners((current) => [...current, owner]);
    return () => setOwners((current) => current.filter(({ id }) => id !== owner.id));
  }, []);
  useEffect(() => {
    if (phase !== "showing" || focused) return;
    const timer = window.setTimeout(() => setPhase("finished"), 5000);
    return () => window.clearTimeout(timer);
  }, [phase, focused]);
  const visibleOwner = owners[owners.length - 1];
  const action = visibleOwner ? visibleOwner.rerun : rerun;
  return (
    <RegisterRerun value={register}>
      {phase === "showing" && ready && (
        <GlobalFeedback>
          <div
            onFocusCapture={() => setFocused(true)}
            onBlurCapture={(event) => {
              if (!event.currentTarget.contains(event.relatedTarget)) setFocused(false);
            }}
          >
            <Alert
              banner
              role="status"
              type="success"
              title="Semantic search is ready"
              action={
                action && (
                  <Button
                    size="small"
                    onClick={() => {
                      setPhase("finished");
                      action();
                    }}
                  >
                    Rerun this search
                  </Button>
                )
              }
            />
          </div>
        </GlobalFeedback>
      )}
      {children}
    </RegisterRerun>
  );
}
