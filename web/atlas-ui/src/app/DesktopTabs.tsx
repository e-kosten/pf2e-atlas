import { ConfigProvider } from "antd";
import { useQueryClient } from "@tanstack/react-query";
import { arrayMove } from "@dnd-kit/sortable";
import { RouterProvider, type ParsedLocation } from "@tanstack/react-router";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { createAtlasRouter } from "./router";
import { createDesktopTabHistory } from "./desktopTabHistory";
import { antDesignTheme, type ResolvedColorScheme } from "../shared/theme/atlasTheme";
import {
  isSettingsPath,
  readDesktopAttachment,
  settingsPath,
  useDesktopAttachment,
  useSettingsRequest,
} from "./desktopBridge";
import { DesktopTabBar, desktopTabButtonId, desktopTabPanelId } from "./DesktopTabBar";
import {
  DesktopTabContext,
  canFocusTabElement,
  type DesktopTabRuntime,
  type TabCloseGuard,
} from "./DesktopTabContext";
import "./desktopTabs.css";

const commandEvent = "atlas-windows-tab-command";

type Command =
  | { kind: "open"; href: string; foreground: boolean; delivery?: number }
  | { kind: "select"; id: string; href: string; delivery: number }
  | { kind: "check-exit"; id: number }
  | { kind: "close-tab" | "close-window" };
type Attachment = { token: string };
type DesktopWindow = Window & {
  __ATLAS_DESKTOP_SETUP__?: Attachment;
  __TAURI_INTERNALS__?: {
    invoke: (name: string, args: Record<string, unknown>) => Promise<unknown>;
  };
};

function invoke(name: string, args: Record<string, unknown> = {}) {
  const host = window as DesktopWindow;
  const token = host.__ATLAS_DESKTOP_SETUP__?.token;
  if (!token) return Promise.resolve();
  return (
    host.__TAURI_INTERNALS__?.invoke(name, { token, ...args }) ?? Promise.resolve()
  );
}

function reportCommandFailure(error: unknown) {
  console.error("Atlas tab command failed", error);
}

function createTab(href: string) {
  let active = false;
  const router = createAtlasRouter(createDesktopTabHistory(href), () => active);
  const scrollByEntry = new Map<string, { x: number; y: number }>();
  const scrollKey = (location: ParsedLocation) =>
    location.state.__TSR_key ?? location.href;
  router.subscribe("onBeforeLoad", ({ fromLocation }) => {
    if (active && fromLocation)
      scrollByEntry.set(scrollKey(fromLocation), {
        x: window.scrollX,
        y: window.scrollY,
      });
  });
  router.subscribe("onRendered", ({ fromLocation, toLocation }) => {
    if (!active || fromLocation?.href === toLocation.href || toLocation.hash) return;
    const position = scrollByEntry.get(scrollKey(toLocation));
    if (position) window.scrollTo(position.x, position.y);
  });
  return {
    id: crypto.randomUUID(),
    settings: isSettingsPath(new URL(href, window.location.href).pathname),
    router,
    setVisible(visible: boolean) {
      active = visible;
    },
    guards: new Set<TabCloseGuard>(),
  };
}

type Tab = ReturnType<typeof createTab>;

function closeStatus(tab: Tab) {
  const guards = Array.from(tab.guards, (guard) => guard());
  if (guards.includes("pending")) return "pending";
  if (guards.includes("dirty")) return "dirty";
}

export function DesktopTabs({ initialHref }: { initialHref: string }) {
  const client = useQueryClient();
  const [first] = useState(() => createTab(initialHref));
  const [tabs, setTabs] = useState([first]);
  const showStrip = tabs.length > 1;
  const [selection, setSelection] = useState<{ id: string; activation: number }>({
    id: first.id,
    activation: 0,
  });
  const active = tabs.find((tab) => tab.id === selection.id)!;
  const [scheme, setScheme] = useState<ResolvedColorScheme>("light");
  const [titles, setTitles] = useState<Record<string, string>>({});
  const strip = useRef<HTMLDivElement>(null);
  const closeFocus = useRef<"tab" | "content" | null>(null);
  const settingsOrigin = useRef<string | null>(null);
  const attachment = useDesktopAttachment();
  const pendingSettingsShortcut = useRef(false);
  const openSettings = useCallback(() => {
    const current = readDesktopAttachment();
    if (!current?.token) {
      pendingSettingsShortcut.current = true;
      return;
    }
    const hostAlreadyRequestedSettings =
      pendingSettingsShortcut.current && current.settingsRequest;
    pendingSettingsShortcut.current = false;
    if (!hostAlreadyRequestedSettings) {
      void invoke("desktop_settings_open", { destination: "general" }).catch(
        reportCommandFailure,
      );
    }
  }, []);
  useEffect(() => {
    if (attachment?.token && pendingSettingsShortcut.current) openSettings();
  }, [attachment?.token, openSettings]);
  const positions = useRef(
    new Map<string, { x: number; y: number; focus: HTMLElement | null }>(),
  );
  const snapshotSequence = useRef(0);
  const [delivery, setDelivery] = useState<{ id: number; accepted: boolean }>();

  const publish = useCallback(() => {
    return invoke("desktop_tabs_state", {
      snapshot: {
        sequence: ++snapshotSequence.current,
        active: active.id,
        activation: selection.activation,
        tabs: tabs.map((tab) => ({ id: tab.id, href: tab.router.latestLocation.href })),
      },
    });
  }, [active.id, selection.activation, tabs]);

  useEffect(() => {
    const stops = tabs.map((tab) =>
      tab.router.subscribe(
        "onResolved",
        () => void publish().catch(reportCommandFailure),
      ),
    );
    const publishOnAttachment = () => void publish().catch(reportCommandFailure);
    window.addEventListener("atlas-desktop-setup", publishOnAttachment);
    window.addEventListener("atlas-opening-committed", publishOnAttachment);
    void publish().catch(reportCommandFailure);
    return () => {
      stops.forEach((stop) => stop());
      window.removeEventListener("atlas-desktop-setup", publishOnAttachment);
      window.removeEventListener("atlas-opening-committed", publishOnAttachment);
    };
  }, [publish, tabs]);

  useEffect(() => {
    if (!delivery) return;
    let cancelled = false;
    void publish()
      .then(async () => {
        if (cancelled) return;
        await invoke("desktop_tabs_ack", {
          delivery: delivery.id,
          accepted: delivery.accepted,
        });
        if (!cancelled)
          setDelivery((current) => (current === delivery ? undefined : current));
      })
      .catch(reportCommandFailure);
    return () => {
      cancelled = true;
    };
  }, [delivery, publish]);

  const rememberPosition = useCallback(() => {
    const element = document.activeElement;
    const previous = positions.current.get(active.id);
    positions.current.set(active.id, {
      x: window.scrollX,
      y: window.scrollY,
      focus:
        element instanceof HTMLElement && !strip.current?.contains(element)
          ? element
          : (previous?.focus ?? null),
    });
  }, [active.id]);

  const select = useCallback(
    (id: string) => {
      if (id === active.id) return;
      rememberPosition();
      setSelection((previous) => ({ id, activation: previous.activation + 1 }));
    },
    [active.id, rememberPosition],
  );

  const moveTab = (id: string, target: (tabs: Tab[], index: number) => number) => {
    setTabs((current) => {
      const from = current.findIndex((tab) => tab.id === id);
      if (from < 0) return current;
      const to = target(current, from);
      return to < 0 || to >= current.length || from === to
        ? current
        : arrayMove(current, from, to);
    });
  };

  const open = useCallback(
    (href: string, foreground: boolean) => {
      const url = new URL(href, window.location.href);
      if (url.origin !== window.location.origin) return;
      const tab = createTab(url.pathname + url.search + url.hash);
      setTabs((previous) => [...previous, tab]);
      if (foreground) select(tab.id);
    },
    [select],
  );

  useSettingsRequest(
    attachment?.settingsRequest,
    attachment?.token,
    (destination) => {
      const href = settingsPath(destination, true);
      const existing = tabs.find((tab) => tab.settings);
      if (!active.settings) settingsOrigin.current = active.id;
      if (!existing) {
        open(href, true);
        return;
      }
      select(existing.id);
      if (
        destination !== "general" &&
        existing.router.latestLocation.pathname !== href
      ) {
        return existing.router.navigate({ href });
      }
    },
    reportCommandFailure,
  );

  const close = useCallback(
    (id: string) => {
      const index = tabs.findIndex((tab) => tab.id === id);
      if (index < 0) return false;
      const status = closeStatus(tabs[index]);
      if (status === "pending") {
        window.alert("Wait for the save to finish before closing this tab.");
        return false;
      }
      if (status === "dirty" && !window.confirm("Discard unsaved changes in this tab?"))
        return false;
      if (tabs.length === 1) {
        void invoke("desktop_tabs_close").catch(reportCommandFailure);
        return true;
      }
      if (id === active.id) {
        const closingSettings = tabs[index].settings;
        const origin = closingSettings
          ? tabs.find((tab) => tab.id === settingsOrigin.current)
          : undefined;
        closeFocus.current = closingSettings ? "content" : "tab";
        select(origin?.id ?? tabs[index + 1]?.id ?? tabs[index - 1].id);
      } else if (tabs.length === 2 && strip.current?.contains(document.activeElement)) {
        document.getElementById("atlas-main")?.focus({ preventScroll: true });
      }
      positions.current.delete(id);
      setTabs((previous) => previous.filter((tab) => tab.id !== id));
      setTitles((previous) => {
        const next = { ...previous };
        delete next[id];
        return next;
      });
      return true;
    },
    [active.id, select, tabs],
  );

  const closeWindow = useCallback(() => {
    if (tabs.some((tab) => closeStatus(tab) === "pending")) {
      window.alert("Wait for the save to finish before closing this window.");
      return;
    }
    if (
      tabs.some((tab) => closeStatus(tab) === "dirty") &&
      !window.confirm("Discard unsaved changes in this window?")
    )
      return;
    void invoke("desktop_tabs_close").catch(reportCommandFailure);
  }, [tabs]);

  useLayoutEffect(() => {
    const focusAtSelection = document.activeElement;
    const frame = requestAnimationFrame(() => {
      const position = positions.current.get(active.id);
      const { x, y } = position ?? { x: 0, y: 0 };
      if (window.scrollX !== x || window.scrollY !== y) window.scrollTo(x, y);
      const selected = strip.current?.querySelector<HTMLElement>(
        '[role="tab"][aria-selected="true"]',
      );
      if (closeFocus.current) {
        const restoreContent = closeFocus.current === "content";
        closeFocus.current = null;
        const content = canFocusTabElement(position?.focus)
          ? position.focus
          : document.getElementById("atlas-main");
        (restoreContent ? (content ?? selected) : (selected ?? content))?.focus({
          preventScroll: true,
        });
      } else if (strip.current?.contains(document.activeElement)) {
        selected?.focus({ preventScroll: true });
      } else if (
        focusAtSelection === document.activeElement ||
        !document
          .getElementById(desktopTabPanelId(active.id))
          ?.contains(document.activeElement)
      ) {
        // A resumed editor or its user can focus a control before this frame.
        const focus = canFocusTabElement(position?.focus) ? position.focus : selected;
        focus?.focus({ preventScroll: true });
      }
    });
    return () => cancelAnimationFrame(frame);
  }, [active]);

  useLayoutEffect(() => {
    function receive(event: Event) {
      const command = (event as CustomEvent<Command>).detail;
      switch (command.kind) {
        case "open":
          open(command.href, command.foreground);
          if (command.delivery !== undefined)
            setDelivery({ id: command.delivery, accepted: true });
          break;
        case "select":
          if (
            !tabs.some(
              (tab) =>
                tab.id === command.id &&
                tab.router.latestLocation.href === command.href,
            )
          ) {
            setDelivery({ id: command.delivery, accepted: false });
            return;
          }
          select(command.id);
          setDelivery({ id: command.delivery, accepted: true });
          break;
        case "check-exit": {
          const statuses = tabs.map(closeStatus);
          void invoke("desktop_tabs_exit_status", {
            id: command.id,
            dirty: statuses.includes("dirty"),
            pending: statuses.includes("pending") || client.isMutating() > 0,
          }).catch(reportCommandFailure);
          break;
        }
        case "close-tab":
          close(active.id);
          break;
        case "close-window":
          closeWindow();
          break;
      }
    }

    function keydown(event: KeyboardEvent) {
      if (event.defaultPrevented || event.isComposing) return;
      if (event.altKey && !event.ctrlKey && !event.shiftKey && !event.metaKey) {
        if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
          event.preventDefault();
          if (event.key === "ArrowLeft") active.router.history.back();
          else active.router.history.forward();
        }
        return;
      }
      if (!event.ctrlKey || event.altKey || event.metaKey) return;
      if (event.key === "Tab") {
        event.preventDefault();
        const index = tabs.indexOf(active);
        const offset = event.shiftKey ? -1 : 1;
        const next = tabs[(index + offset + tabs.length) % tabs.length].id;
        select(next);
        if (strip.current?.contains(document.activeElement)) {
          document
            .getElementById(desktopTabButtonId(next))
            ?.focus({ preventScroll: true });
        }
      } else if (!event.shiftKey) {
        const key = event.key.toLowerCase();
        if (key !== "t" && key !== "w" && key !== "n" && key !== ",") return;
        event.preventDefault();
        if (event.repeat) return;
        if (key === "t") open("/search", true);
        else if (key === "w") close(active.id);
        else if (key === "n")
          void invoke("desktop_tabs_new_window").catch(reportCommandFailure);
        else openSettings();
      }
    }

    const preventReload = (event: BeforeUnloadEvent) => {
      if (tabs.some(closeStatus)) event.preventDefault();
    };
    window.addEventListener("beforeunload", preventReload);
    window.addEventListener(commandEvent, receive);
    window.addEventListener("keydown", keydown, true);
    return () => {
      window.removeEventListener("beforeunload", preventReload);
      window.removeEventListener(commandEvent, receive);
      window.removeEventListener("keydown", keydown, true);
    };
  }, [active, client, close, closeWindow, open, openSettings, select, tabs]);

  const title = useCallback((id: string, name: string) => {
    setTitles((previous) =>
      previous[id] === name ? previous : { ...previous, [id]: name },
    );
  }, []);

  return (
    <ConfigProvider theme={antDesignTheme(scheme)}>
      <div className="desktop-tab-host" data-show-tab-strip={showStrip}>
        {showStrip && (
          <div className="desktop-tab-strip" ref={strip}>
            <DesktopTabBar
              tabs={tabs.map(({ id }) => ({ id, title: titles[id] ?? "Atlas" }))}
              activeId={active.id}
              select={select}
              close={close}
              open={() => open("/search", true)}
              moveBy={(id, offset) => moveTab(id, (_items, from) => from + offset)}
              moveTo={(id, target) =>
                moveTab(id, (items) => items.findIndex((tab) => tab.id === target))
              }
            />
          </div>
        )}
        {tabs.map((tab) => (
          <TabPanel
            key={tab.id}
            tab={tab}
            active={tab === active}
            tabbed={showStrip}
            activation={selection.activation}
            title={title}
            theme={setScheme}
          />
        ))}
      </div>
    </ConfigProvider>
  );
}

function TabPanel({
  tab,
  active,
  tabbed,
  activation,
  title,
  theme,
}: {
  tab: Tab;
  active: boolean;
  tabbed: boolean;
  activation: number;
  title: (id: string, name: string) => void;
  theme: (scheme: ResolvedColorScheme) => void;
}) {
  const panel = useRef<HTMLElement>(null);
  const portal = useCallback(() => panel.current ?? document.body, []);
  useLayoutEffect(() => {
    tab.setVisible(active);
  }, [active, tab]);
  const guard = useCallback(
    (read: TabCloseGuard) => {
      tab.guards.add(read);

      return () => {
        tab.guards.delete(read);
      };
    },
    [tab],
  );
  const runtime = useMemo<DesktopTabRuntime>(
    () => ({
      id: tab.id,
      active,
      activation,
      panel: portal,
      title: (name) => title(tab.id, name),
      theme,
      guard,
    }),
    [activation, active, guard, portal, tab, theme, title],
  );

  return (
    <section
      ref={panel}
      id={desktopTabPanelId(tab.id)}
      role={tabbed ? "tabpanel" : undefined}
      aria-labelledby={tabbed ? desktopTabButtonId(tab.id) : undefined}
      hidden={!active}
      inert={!active}
      className="desktop-tab-page"
    >
      <DesktopTabContext.Provider value={runtime}>
        <ConfigProvider getPopupContainer={portal}>
          <RouterProvider router={tab.router} />
        </ConfigProvider>
      </DesktopTabContext.Provider>
    </section>
  );
}
