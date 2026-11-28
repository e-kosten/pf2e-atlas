import type { RouterHistory } from "@tanstack/react-router";
import {
  invokeDesktop,
  macHistoryEnabled,
  readDesktopAttachment,
  type HistoryDirection,
} from "./desktopBridge";

type Snapshot = { back: boolean; forward: boolean };
type Request =
  | { action: "status" }
  | { action: "go"; direction: HistoryDirection }
  | { action: "finish"; sequence: number };

const empty: Snapshot = { back: false, forward: false };
let snapshot = empty;
let sequence = 0;
let applied = 0;
const listeners = new Set<() => void>();

function publish(next: Snapshot) {
  if (next.back === snapshot.back && next.forward === snapshot.forward) return;
  snapshot = next;
  listeners.forEach((listener) => listener());
}

export const macHistory = {
  getSnapshot: () => snapshot,
  subscribe(listener: () => void) {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
  traverse(direction: HistoryDirection) {
    if (snapshot[direction]) void request({ action: "go", direction }).catch(report);
  },
};

async function request(request: Request, token = readDesktopAttachment()?.token) {
  if (!token || token !== readDesktopAttachment()?.token) return undefined;
  const order = ++sequence;
  const result = await invokeDesktop<Snapshot>("desktop_history", { token, request });
  if (token !== readDesktopAttachment()?.token) return undefined;
  if (order > applied) {
    applied = order;
    publish(result);
  }
  return result;
}

function report(error: unknown) {
  console.error("Atlas history command failed", error);
}

export function connectMacHistory(history: RouterHistory) {
  if (!macHistoryEnabled()) return;

  type Pop = {
    token: string;
    key: string | undefined;
    index: number | undefined;
    href: string;
    sequence: number;
  };
  let pop: Pop | undefined;
  let nextPop = 0;
  let attached: string | undefined;

  const acknowledge = () => {
    const event = pop;
    const state = history.location.state;
    if (
      !event ||
      !(event.index !== undefined
        ? event.index === state.__TSR_index && event.key === state.__TSR_key
        : event.href === history.location.href)
    )
      return;
    pop = undefined;
    const complete = { action: "finish", sequence: event.sequence } as const;
    // The host acknowledges each observed pop once. A lost reply can be
    // retried even if a later traversal has already begun.
    const send = (delay: number) => {
      void request(complete, event.token).catch((error) => {
        if (readDesktopAttachment()?.token !== event.token) return;
        if (delay === 1000) report(error);
        window.setTimeout(() => send(Math.min(delay * 2, 8000)), delay);
      });
    };
    send(1000);
  };

  window.addEventListener(
    "popstate",
    (event) => {
      const token = readDesktopAttachment()?.token;
      if (!token) return;
      pop = {
        token,
        key: event.state?.__TSR_key,
        index: event.state?.__TSR_index,
        href: window.location.pathname + window.location.search + window.location.hash,
        sequence: ++nextPop,
      };
      void request({ action: "status" }, token).catch(report);
      // TanStack consumes the compensating pop without notifying subscribers.
      queueMicrotask(acknowledge);
    },
    { capture: true },
  );

  history.subscribe(({ action }) => {
    if (action.type === "PUSH" || action.type === "REPLACE") {
      queueMicrotask(() => {
        history.flush();
        void request({ action: "status" }).catch(report);
      });
    } else {
      acknowledge();
    }
  });

  // Public app commands go through the native gate. TanStack's private
  // compensation continues to use the underlying History API directly.
  history.canGoBack = () => snapshot.back;
  history.back = () => macHistory.traverse("back");
  history.forward = () => macHistory.traverse("forward");

  const attach = () => {
    const token = readDesktopAttachment()?.token;
    if (token === attached) return;
    attached = token;
    pop = undefined;
    publish(empty);
    void request({ action: "status" }, token).catch(report);
  };
  window.addEventListener("atlas-desktop-setup", attach);
  attach();
  // The primary workspace document can attach before startup admits its render.
  window.addEventListener("atlas-opening-committed", () => {
    void request({ action: "status" }).catch(report);
  });
}
