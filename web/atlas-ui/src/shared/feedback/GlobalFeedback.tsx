import {
  createContext,
  useCallback,
  useContext,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
  type Ref,
} from "react";
import { createPortal } from "react-dom";

type RegisterHost = (element: HTMLElement, depth: number) => () => void;
const RegisterContext = createContext<RegisterHost | null>(null);
const HostContext = createContext<HTMLElement | null>(null);
const DepthContext = createContext(0);
// Ant may cache modalRender children while closed; keep visibility live outside it.
export const FeedbackOverlayActive = createContext<boolean | undefined>(undefined);

/** A shared failure has one visible owner, including inside the active dialog. */
export function GlobalFeedbackProvider({ children }: { children: ReactNode }) {
  const [hosts, setHosts] = useState<
    Array<{ id: symbol; element: HTMLElement; depth: number }>
  >([]);
  const register = useCallback<RegisterHost>((element, depth) => {
    const id = Symbol();
    setHosts((current) => [...current, { id, element, depth }]);
    return () => setHosts((current) => current.filter((host) => host.id !== id));
  }, []);
  const active = hosts.reduce<(typeof hosts)[number] | undefined>(
    (current, host) => (!current || host.depth >= current.depth ? host : current),
    undefined,
  );
  return (
    <RegisterContext value={register}>
      <HostContext value={active?.element ?? null}>{children}</HostContext>
    </RegisterContext>
  );
}

export function GlobalFeedbackHost({
  active,
  sticky = false,
}: {
  active: boolean;
  sticky?: boolean;
}) {
  const overlayActive = useContext(FeedbackOverlayActive);
  const enabled = overlayActive ?? active;
  const register = useContext(RegisterContext);
  const depth = useContext(DepthContext);
  const element = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    if (enabled && element.current && register) return register(element.current, depth);
  }, [enabled, register, depth]);
  return (
    <div
      ref={element}
      className={`global-feedback-host${sticky ? " global-feedback-host--sticky" : ""}`}
    />
  );
}

export function GlobalFeedback({ children }: { children: ReactNode }) {
  const host = useContext(HostContext);
  return host ? createPortal(children, host) : children;
}

export function FeedbackModalFrame({
  active,
  children,
  frameRef,
  fill = false,
}: {
  active: boolean;
  children: ReactNode;
  frameRef?: Ref<HTMLDivElement>;
  fill?: boolean;
}) {
  const depth = useContext(DepthContext);
  return (
    <DepthContext value={depth + 1}>
      <div
        ref={frameRef}
        className={`feedback-modal-layout${fill ? " feedback-modal-layout--fill" : ""}`}
      >
        <GlobalFeedbackHost active={active} />
        {children}
      </div>
    </DepthContext>
  );
}

export function FeedbackDrawerBody({
  active,
  children,
}: {
  active: boolean;
  children: ReactNode;
}) {
  const depth = useContext(DepthContext);
  return (
    <DepthContext value={depth + 1}>
      <GlobalFeedbackHost active={active} sticky />
      {children}
    </DepthContext>
  );
}
