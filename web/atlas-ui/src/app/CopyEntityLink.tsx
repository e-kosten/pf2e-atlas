import { Popover, Tooltip } from "antd";
import { useRouter, useRouterState } from "@tanstack/react-router";
import { Check, Link } from "lucide-react";
import type { ReactNode } from "react";
import { PaneIconButton } from "../shared/ui/actions/PaneAction";
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

type CopyEntityLinkProps = {
  kind: Kind;
  identity: string;
  title: string;
  primary?: boolean;
  actions?: (copy: (() => void) | undefined) => ReactNode;
};

export function CopyEntityLink(props: CopyEntityLinkProps) {
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
  actions,
}: CopyEntityLinkProps) {
  const attachment = useSyncExternalStore(subscribe, token);
  const desktop = Boolean(host().__TAURI__);
  const [hintOpen, setHintOpen] = useState(false);
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

  const inMenu = Boolean(actions);
  useEffect(() => {
    if (result !== "copied") return;
    const timeout = window.setTimeout(() => setResult(undefined), inMenu ? 1600 : 800);
    return () => window.clearTimeout(timeout);
  }, [inMenu, result]);

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

  const available = !desktop || Boolean(attachment);
  const Button = kind === "record" ? PaneIconButton : InlineIconButton;
  const Icon = result === "copied" ? Check : Link;
  const copyLink = () => void copy();

  const trigger = actions
    ? actions(available ? copyLink : undefined)
    : available && (
        <Button
          {...(kind === "record" ? { outlined: true } : {})}
          label="Copy link"
          tooltip={false}
          icon={<Icon size={16} aria-hidden="true" />}
          onClick={copyLink}
        />
      );

  return (
    <>
      <Popover
        open={result === "failed" && !attachment}
        onOpenChange={(open) => {
          if (!open) setResult(undefined);
        }}
        trigger="click"
        title="Could not copy link"
        content={
          <input
            aria-label="Link to copy"
            readOnly
            value={url}
            onFocus={(event) => event.currentTarget.select()}
          />
        }
      >
        <span style={{ display: "inline-flex" }}>
          <Tooltip
            trigger={actions ? [] : ["hover", "focus"]}
            open={
              result === "copied" ||
              (result === "failed" && Boolean(attachment)) ||
              (!actions && !result && hintOpen)
            }
            onOpenChange={setHintOpen}
            title={
              result === "copied"
                ? actions
                  ? "Link copied"
                  : "Copied"
                : result === "failed"
                  ? "Could not copy link"
                  : "Copy link"
            }
          >
            <span style={{ display: "inline-flex" }}>{trigger}</span>
          </Tooltip>
        </span>
      </Popover>
      <span className="sr-only" aria-live="polite">
        {result === "copied" && "Link copied"}
        {result === "failed" && "Could not copy link"}
      </span>
    </>
  );
}
