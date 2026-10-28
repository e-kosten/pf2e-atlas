import { Link, useRouter } from "@tanstack/react-router";
import { AtlasLink } from "./AtlasLink";
import {
  searchDestination,
  listsDestination,
  encountersDestination,
} from "./destinations";
import { Monitor, Moon, Sun } from "lucide-react";
import { useRef, useSyncExternalStore } from "react";
import {
  getCompatibilityFailure,
  subscribeCompatibilityFailures,
} from "../api/compatibility";
import { reloadFrontend } from "../shared/browser/reloadFrontend";
import {
  AtlasConnectionError,
  getAvailabilityFailure,
  subscribeAvailabilityFailures,
} from "../api/availability";
import { errorExplanation } from "../shared/feedback/errorMessages";
import { GlobalFeedback } from "../shared/feedback/GlobalFeedback";
import { Alert, Button, Menu, Popover, Segmented, Space, Typography } from "antd";
import type { MenuProps } from "antd";
import { restoreOverlayFocus } from "../shared/ui/overlays/restoreOverlayFocus";
import type { AppReadinessView } from "../generated/atlas";
import type {
  ColorSchemePreference,
  ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { AtlasMark } from "../shared/branding/AtlasMark";

type AppShellProps = {
  activeView: "search" | "lists" | "encounters" | "reference";
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
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
  resolvedColorScheme,
  readiness,
  readinessError,
  checkingReadiness,
  onCheckReadiness,
  children,
}: AppShellProps) {
  const router = useRouter();
  const contentRef = useRef<HTMLDivElement>(null);
  const themeTrigger = useRef<HTMLButtonElement>(null);
  const themeContent = useRef<HTMLDivElement>(null);
  const selectedView = activeView;
  const compatibilityFailure = useSyncExternalStore(
    subscribeCompatibilityFailures,
    getCompatibilityFailure,
  );
  const availabilityFailure = useSyncExternalStore(
    subscribeAvailabilityFailures,
    getAvailabilityFailure,
  );
  const connectionFailure =
    availabilityFailure instanceof AtlasConnectionError ? availabilityFailure : null;
  const destinations = [
    {
      key: "search",
      label: "Search",
      destination: searchDestination(),
    },
    {
      key: "lists",
      label: "Lists",
      destination: listsDestination(),
    },
    {
      key: "encounters",
      label: "Encounters",
      destination: encountersDestination(),
    },
  ];
  const navItems: MenuProps["items"] = destinations.map((destination) => ({
    key: destination.key,
    label: (
      <Link
        {...destination.destination}
        aria-current={selectedView === destination.key ? "page" : undefined}
        activeProps={{ "aria-current": undefined }}
      >
        {destination.label}
      </Link>
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
    const destination = destinations.find(
      (destination) => destination.key === key,
    )?.destination;
    if (destination) void router.navigate(destination);
  };
  const availabilityError =
    availabilityFailure || readinessError || readiness?.status === "blocked";
  const connectionLost =
    !!connectionFailure || readinessError instanceof AtlasConnectionError;
  const availabilityTitle = connectionLost
    ? "Unable to connect to Atlas"
    : readinessError
      ? "Atlas unavailable"
      : "Reference unavailable";
  const availabilityHelp = connectionLost
    ? undefined
    : availabilityFailure || readiness?.status === "blocked"
      ? "Check Atlas setup before retrying."
      : errorExplanation(readinessError);
  return (
    <div
      className={`atlas-app${activeView === "reference" ? " atlas-app--document" : activeView === "search" ? " atlas-app--search" : ""}`}
      data-theme={resolvedColorScheme}
    >
      <Typography.Link
        className="skip-link"
        href="#atlas-main"
        onClick={() => contentRef.current?.focus()}
      >
        Skip to main content
      </Typography.Link>
      <header className="topbar">
        <AtlasLink
          {...searchDestination()}
          className="topbar__brand"
          style={{ color: "var(--brand-text)" }}
          aria-label="PF2e Atlas"
        >
          <AtlasMark />
          <span>PF2e Atlas</span>
        </AtlasLink>
        <Menu
          aria-label="Atlas views"
          className="topbar__nav"
          disabledOverflow
          items={navItems}
          mode="horizontal"
          onClick={onNavClick}
          selectedKeys={[selectedView]}
          theme={resolvedColorScheme}
        />
        <Space className="topbar__actions" size="small">
          <Popover
            trigger="click"
            onOpenChange={(open) => {
              if (!open)
                restoreOverlayFocus(themeTrigger.current, themeContent.current);
            }}
            placement="bottom"
            title="Theme"
            content={
              <div ref={themeContent}>
                <Space orientation="vertical" size="small">
                  <Segmented
                    aria-label="Appearance"
                    value={colorScheme}
                    onChange={onColorSchemeChange}
                    options={[
                      { value: "system", label: "System" },
                      { value: "light", label: "Light" },
                      { value: "dark", label: "Dark" },
                    ]}
                  />
                </Space>
              </div>
            }
          >
            <Button
              ref={themeTrigger}
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
      <GlobalFeedback>
        {compatibilityFailure && (
          <Alert
            className="error-banner"
            type="warning"
            role="status"
            banner
            showIcon
            title={
              <>
                Reload Atlas to continue.{" "}
                <Typography.Text type="secondary">
                  Copy any unsaved edits first.
                </Typography.Text>
              </>
            }
            action={
              <Button size="small" onClick={reloadFrontend}>
                Reload page
              </Button>
            }
          />
        )}
        {availabilityError && !compatibilityFailure && (
          <Alert
            className="error-banner"
            type={connectionLost || readinessError ? "error" : "warning"}
            role={connectionLost || readinessError ? "alert" : "status"}
            showIcon
            banner
            title={
              <>
                {availabilityTitle}
                {availabilityHelp && (
                  <>
                    .{" "}
                    <Typography.Text type="secondary">
                      {availabilityHelp}
                    </Typography.Text>
                  </>
                )}
              </>
            }
            action={
              <Button
                size="small"
                loading={checkingReadiness}
                onClick={onCheckReadiness}
              >
                Retry
              </Button>
            }
          />
        )}
      </GlobalFeedback>
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
