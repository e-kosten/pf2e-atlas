import { Monitor, Moon, Sun } from "lucide-react";
import { useRef } from "react";
import { Alert, Button, Menu, Popover, Segmented, Space, Typography } from "antd";
import type { MenuProps } from "antd";
import type { AppReadinessView } from "../generated/atlas";
import type {
  ColorSchemePreference,
  ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import {
  encountersPath,
  listsPath,
  searchPath,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "./routes";

type AppShellProps = {
  activeView: AtlasRoute["kind"];
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
  onNavigateLists: () => void;
  onNavigateEncounters: () => void;
  onNavigateSearch: () => void;
  resolvedColorScheme: ResolvedColorScheme;
  readiness: AppReadinessView | undefined;
  readinessError: Error | null;
  checkingReadiness: boolean;
  onCheckReadiness: () => void;
  children: React.ReactNode;
};

export function AppShell({
  activeView,
  colorScheme,
  onColorSchemeChange,
  onNavigateLists,
  onNavigateEncounters,
  onNavigateSearch,
  resolvedColorScheme,
  readiness,
  readinessError,
  checkingReadiness,
  onCheckReadiness,
  children,
}: AppShellProps) {
  const contentRef = useRef<HTMLDivElement>(null);
  const selectedView = topLevelView(activeView);
  const destinations = [
    { key: "search", label: "Search", href: searchPath(), navigate: onNavigateSearch },
    { key: "lists", label: "Lists", href: listsPath(), navigate: onNavigateLists },
    {
      key: "encounters",
      label: "Encounters",
      href: encountersPath(),
      navigate: onNavigateEncounters,
    },
  ];
  const navItems: MenuProps["items"] = destinations.map((destination) => ({
    key: destination.key,
    label: (
      <a
        href={destination.href}
        aria-current={selectedView === destination.key ? "page" : undefined}
        onClick={(event) => {
          if (!shouldHandleAtlasRouteClick(event)) return;
          event.preventDefault();
          destination.navigate();
        }}
      >
        {destination.label}
      </a>
    ),
  }));
  const onNavClick: MenuProps["onClick"] = ({ key, domEvent }) => {
    if (
      domEvent.type !== "keydown" ||
      (domEvent.target instanceof Element && domEvent.target.closest("a")) ||
      domEvent.metaKey ||
      domEvent.ctrlKey ||
      domEvent.altKey ||
      domEvent.shiftKey
    )
      return;
    destinations.find((destination) => destination.key === key)?.navigate();
  };
  const availabilityError =
    readinessError?.message ??
    (readiness?.status === "blocked" ? readiness.message : null);
  return (
    <div className="atlas-app" data-theme={resolvedColorScheme}>
      <Typography.Link
        className="skip-link"
        href="#atlas-main"
        onClick={() => contentRef.current?.focus()}
      >
        Skip to main content
      </Typography.Link>
      <header className="topbar">
        <Typography.Link
          className="topbar__brand"
          href={searchPath()}
          onClick={(event) => {
            if (!shouldHandleAtlasRouteClick(event)) return;
            event.preventDefault();
            onNavigateSearch();
          }}
        >
          <span className="topbar__brand-prefix">PF2e </span>Atlas
        </Typography.Link>
        <Menu
          aria-label="Atlas views"
          className="topbar__nav"
          disabledOverflow
          items={navItems}
          mode="horizontal"
          onClick={onNavClick}
          selectedKeys={[selectedView]}
        />
        <Space className="topbar__actions" size="small">
          <Popover
            trigger="click"
            placement="bottom"
            title="Theme"
            content={
              <Segmented
                value={colorScheme}
                onChange={onColorSchemeChange}
                options={[
                  { value: "system", label: "System" },
                  { value: "light", label: "Light" },
                  { value: "dark", label: "Dark" },
                ]}
              />
            }
          >
            <Button
              type="text"
              aria-label="Theme"
              icon={
                colorScheme === "system" ? (
                  <Monitor size={18} />
                ) : colorScheme === "light" ? (
                  <Sun size={18} />
                ) : (
                  <Moon size={18} />
                )
              }
            />
          </Popover>
        </Space>
      </header>
      {availabilityError && (
        <Alert
          className="error-banner"
          type="error"
          showIcon
          title={
            readinessError ? "Unable to connect to Atlas" : "Reference unavailable"
          }
          description={availabilityError}
          action={
            <Button loading={checkingReadiness} onClick={onCheckReadiness}>
              Retry
            </Button>
          }
        />
      )}
      <div
        className="atlas-app__content"
        id="atlas-main"
        ref={contentRef}
        tabIndex={-1}
      >
        {children}
      </div>
    </div>
  );
}

function topLevelView(activeView: AtlasRoute["kind"]): string {
  if (activeView === "encounter" || activeView === "encounterEdit") return "encounters";
  if (activeView === "list" || activeView === "listEdit") return "lists";
  return activeView;
}
