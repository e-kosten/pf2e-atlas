import { useRouter, useRouterState } from "@tanstack/react-router";
import { Link } from "lucide-react";
import { useEffect, useState, useSyncExternalStore } from "react";
import { InlineIconButton } from "../shared/ui/actions/InlineAction";
import {
  creatureFamilyDestination,
  definitionDestination,
  encounterDestination,
  equipmentFamilyDestination,
  listDestination,
  recordDestination,
  traitDestination,
} from "./destinations";
import { destinationHref } from "./navigation";
import { atlasRouter } from "./router";

const destinations = {
  record: (key: string) => recordDestination({ recordKey: key }),
  list: (key: string) => listDestination({ slug: key, selectedRecordKey: null }),
  encounter: (key: string) => encounterDestination({ slug: key }),
  trait: (key: string) => traitDestination({ traitKey: key }),
  definition: (key: string) => definitionDestination({ definitionKey: key }),
  "creature-family": creatureFamilyDestination,
  "equipment-family": equipmentFamilyDestination,
};

type Kind = keyof typeof destinations;
type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: { token: string };
  __TAURI__?: { core: { invoke: (command: string, args: object) => Promise<unknown> } };
};

function host() {
  return window as DesktopWindow;
}

let availabilitySequence = 0;

function token() {
  return host().__ATLAS_DESKTOP_SETUP__?.token;
}

function subscribe(callback: () => void) {
  window.addEventListener("atlas-desktop-setup", callback);
  return () => window.removeEventListener("atlas-desktop-setup", callback);
}

function invoke(command: string, args: object) {
  const core = host().__TAURI__?.core;
  return core
    ? core.invoke(command, args)
    : Promise.reject(new Error("Atlas is unavailable."));
}

export function CopyEntityLink(props: {
  kind: Kind;
  identity: string;
  title: string;
  primary?: boolean;
}) {
  const href = useRouterState({
    router: useRouter({ warn: false }) ?? atlasRouter,
    select: (state) => state.location.href,
  });
  return (
    <EntityCopyAction
      key={JSON.stringify([href, props.kind, props.identity])}
      {...props}
    />
  );
}

function EntityCopyAction({
  kind,
  identity,
  title,
  primary,
}: {
  kind: Kind;
  identity: string;
  title: string;
  primary?: boolean;
}) {
  const attachment = useSyncExternalStore(subscribe, token);
  const desktop = Boolean(host().__TAURI__);
  const [result, setResult] = useState<"copied" | "failed">();
  const route = window.location.href;
  const url = new URL(
    destinationHref(destinations[kind](identity)),
    window.location.origin,
  ).href;

  useEffect(() => {
    if (!attachment || !primary) return;
    const args = { token: attachment, route };
    const receive = (event: Event) => {
      if (window.location.href !== route) return;
      void invoke("desktop_copy_link", {
        ...args,
        kind,
        key: identity,
        title,
        request: (event as CustomEvent<number>).detail,
      }).catch(() => setResult("failed"));
    };
    window.addEventListener("atlas-copy-link", receive);
    const publish = (available: boolean) =>
      void invoke("desktop_copy_availability", {
        ...args,
        available,
        sequence: ++availabilitySequence,
      }).catch(() => {});
    publish(true);
    return () => {
      window.removeEventListener("atlas-copy-link", receive);
      publish(false);
    };
  }, [attachment, primary, route, kind, identity, title]);

  useEffect(() => {
    if (result !== "copied") return;
    const timeout = window.setTimeout(() => setResult(undefined), 1600);
    return () => window.clearTimeout(timeout);
  }, [result]);

  async function copy() {
    try {
      if (attachment) {
        await invoke("desktop_copy_link", {
          token: attachment,
          route,
          kind,
          key: identity,
          title,
        });
      } else {
        await navigator.clipboard.writeText(url);
      }
      setResult("copied");
    } catch {
      setResult("failed");
    }
  }

  if (desktop && !attachment) return null;

  return (
    <>
      <InlineIconButton
        label={`Copy ${kind.replace("-", " ")} link`}
        icon={<Link size={16} aria-hidden="true" />}
        onClick={() => void copy()}
      />
      <span aria-live="polite">
        {result === "copied" && "Link copied"}
        {result === "failed" &&
          (attachment ? (
            "Could not copy link"
          ) : (
            <>
              Could not copy link.{" "}
              <input
                aria-label="Link to copy"
                readOnly
                value={url}
                onFocus={(event) => event.currentTarget.select()}
              />
            </>
          ))}
      </span>
    </>
  );
}
