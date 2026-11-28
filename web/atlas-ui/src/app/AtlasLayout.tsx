import { useDesktopTab } from "./DesktopTabContext";
import { useAtlasRouter } from "./router";
import { ConfigProvider, App } from "antd";
import {
  Activity,
  Suspense,
  lazy,
  useEffect,
  useLayoutEffect,
  useState,
  type ReactNode,
} from "react";
import { Outlet, useRouterState } from "@tanstack/react-router";
import {
  COLOR_SCHEME_STORAGE_KEY,
  applyAtlasDocumentTheme,
  atlasPopoverStyles,
  atlasButtonClassNames,
  atlasRadioStyles,
  antDesignTheme,
  type ColorSchemePreference,
  type ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { isSettingsPath } from "./desktopBridge";
import { AppShell } from "./AppShell";
import { currentSearchLocation } from "./searchLocation";
import { searchDestination, type SearchPageInput } from "./destinations";
import { LinkPickerContext } from "../shared/filters/LinkPickerContext";
import { LinkConditionPicker } from "../shared/filters/LinkConditionPicker";
import { BackendCompatibilityGate } from "./BackendCompatibilityGate";
import { SemanticReadyNotice } from "../shared/search/SemanticReadyNotice";
import { SavedWritesPaused } from "../shared/feedback/SavedWritesPaused";
import { GlobalFeedbackProvider } from "../shared/feedback/GlobalFeedback";
import { ReferenceHelpHost } from "../shared/records/ReferenceHelpHost";
import { AddToRecordButton } from "../features/record-actions/AddToRecordButton";
import { SelectionSearch } from "../shared/records/SelectionSearch";
import { LoadingStatus } from "../shared/ui/LoadingStatus";

import { useSearchWorkspace } from "../features/search/useSearchWorkspace";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { isReferenceQuery } from "../shared/queries/referenceQueries";

import { getReadiness } from "../api/atlasApi";
import {
  SearchProfileContext,
  useSearchProfileSelection,
} from "../shared/search/SearchProfileContext";
const SearchView = lazy(() =>
  import("../features/search/SearchView").then((module) => ({
    default: module.SearchView,
  })),
);

export function AtlasLayout({ routeError }: { routeError?: ReactNode } = {}) {
  const settings = useRouterState({
    select: (state) => isSettingsPath(state.location.pathname) && "__TAURI__" in window,
  });
  const tab = useDesktopTab();
  const active = tab?.active ?? true;
  const [colorScheme, setColorScheme] =
    useState<ColorSchemePreference>(readStoredColorScheme);
  const systemColorScheme = useSystemColorScheme();
  const resolvedColorScheme =
    colorScheme === "system" ? systemColorScheme : colorScheme;
  const reportTheme = tab?.theme;
  useLayoutEffect(() => {
    if (!active) return;
    applyAtlasDocumentTheme(document, resolvedColorScheme);
    reportTheme?.(resolvedColorScheme);
  }, [active, reportTheme, resolvedColorScheme]);
  useEffect(() => {
    if (!active) return;
    window.dispatchEvent(
      new CustomEvent("atlas-appearance-changed", {
        detail: { appearance: colorScheme, dark: resolvedColorScheme === "dark" },
      }),
    );
  }, [active, colorScheme, resolvedColorScheme]);
  useEffect(() => {
    const sync = (event: StorageEvent) => {
      if (event.key === COLOR_SCHEME_STORAGE_KEY || event.key === null) {
        setColorScheme(readStoredColorScheme());
      }
    };
    window.addEventListener("storage", sync);
    return () => window.removeEventListener("storage", sync);
  }, []);
  const updateColorScheme = (preference: ColorSchemePreference) => {
    setColorScheme(preference);
    try {
      localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, preference);
      if (tab)
        window.dispatchEvent(
          new StorageEvent("storage", { key: COLOR_SCHEME_STORAGE_KEY }),
        );
    } catch {
      // Browser preferences are optional, including when storage is unavailable.
    }
  };
  return (
    <ConfigProvider
      theme={antDesignTheme(resolvedColorScheme)}
      textArea={{
        styles: {
          textarea: {
            // Autosizing sets the measured height while the browser tracks the caret.
            transitionProperty: "background-color, border-color, box-shadow, color",
          },
        },
      }}
      button={{ classNames: atlasButtonClassNames }}
      radio={{ styles: atlasRadioStyles(resolvedColorScheme) }}
      popover={{
        styles: atlasPopoverStyles(resolvedColorScheme),
      }}
      select={{
        classNames: {
          popup: { root: "atlas-select-popup", listItem: "atlas-select-option" },
        },
      }}
    >
      <GlobalFeedbackProvider>
        <App component={false}>
          <BackendCompatibilityGate>
            {settings && !routeError ? (
              <Outlet />
            ) : (
              <AtlasWorkspace
                colorScheme={colorScheme}
                onColorSchemeChange={updateColorScheme}
                resolvedColorScheme={resolvedColorScheme}
                routeError={routeError}
              />
            )}
          </BackendCompatibilityGate>
        </App>
      </GlobalFeedbackProvider>
    </ConfigProvider>
  );
}

function AtlasWorkspace({
  colorScheme,
  onColorSchemeChange,
  resolvedColorScheme,
  routeError,
}: {
  colorScheme: ColorSchemePreference;
  onColorSchemeChange: (preference: ColorSchemePreference) => void;
  resolvedColorScheme: ResolvedColorScheme;
  routeError?: ReactNode;
}) {
  const atlasRouter = useAtlasRouter();
  const active = useDesktopTab()?.active ?? true;
  const section = useRouterState({
    select: (state) =>
      [...state.matches].reverse().find((match) => match.staticData.section)?.staticData
        .section ?? "search",
  });
  const location = useRouterState({ select: (state) => state.location });
  const [searchReturn, setSearchReturn] = useState<{
    route: SearchPageInput;
    location: typeof location;
  } | null>(() => {
    const route = currentSearchLocation(atlasRouter);
    return !routeError && route ? { route, location } : null;
  });
  if (!routeError && searchReturn?.location !== location) {
    const route = currentSearchLocation(atlasRouter);
    if (route) setSearchReturn({ route, location });
  }
  const queryClient = useQueryClient();
  useEffect(() => {
    if (!active) return;
    const refreshReference = (event: Event) => {
      const semanticOnly = event instanceof CustomEvent && event.detail === "semantic";
      void queryClient.invalidateQueries({
        predicate: (query) =>
          query.queryKey[0] === "readiness" ||
          (!semanticOnly && isReferenceQuery(query)),
      });
    };
    window.addEventListener("atlas-reference-updated", refreshReference);
    return () =>
      window.removeEventListener("atlas-reference-updated", refreshReference);
  }, [active, queryClient]);
  const readiness = useQuery({
    queryKey: ["readiness"],
    queryFn: getReadiness,
    enabled: active,
    refetchInterval: (query) => (query.state.data?.saved_writes_paused ? 1500 : false),
  });
  const searchProfile = useSearchProfileSelection(readiness.data, () =>
    queryClient.fetchQuery({
      queryKey: ["readiness"],
      queryFn: getReadiness,
      staleTime: 0,
    }),
  );
  const workspace = useSearchWorkspace({
    enabled: !routeError && section === "search",
    referenceReady: readiness.data?.status === "ready",
    initialReferenceGeneration: readiness.data?.artifact_generation ?? undefined,
    searchProfile: searchProfile.id,
    pendingUrlProfile: searchProfile.pendingUrlProfile,
    searchProfileStored: searchProfile.storedVectors,
    searchProfileUsable: searchProfile.usable,
    keywordSearch: searchProfile.keywordSearch,
    automaticKeyword: searchProfile.automaticKeyword,
    semanticDefault: searchProfile.semanticDefault,
    promoteKeyword: searchProfile.promoteKeyword,
    resolveWorkspaceDefault: searchProfile.resolveWorkspaceDefault,
    adoptWorkspaceDefault: searchProfile.adoptWorkspaceDefault,
    searchPreferences: searchProfile.preferences,
    includeSearchProfileInUrl:
      searchProfile.profiles.filter((profile) => profile.stored_vectors).length !== 1 ||
      !searchProfile.storedVectors,
  });
  return (
    <SearchProfileContext value={searchProfile}>
      <SavedWritesPaused value={readiness.data?.saved_writes_paused === true}>
        <ReferenceHelpHost
          recordActions={(record) => (
            <AddToRecordButton recordKey={record.record_key} record={record} />
          )}
        >
          <LinkPickerContext value={LinkConditionPicker}>
            <AppShell
              activeView={routeError ? "reference" : section}
              searchReturnDestination={
                searchReturn
                  ? {
                      ...searchDestination(searchReturn.route),
                      hash: searchReturn.location.hash,
                      state: searchReturn.location.state,
                      resetScroll: false,
                    }
                  : undefined
              }
              colorScheme={colorScheme}
              onColorSchemeChange={onColorSchemeChange}
              resolvedColorScheme={resolvedColorScheme}
              readiness={readiness.data}
              readinessError={readiness.error}
              checkingReadiness={readiness.isFetching}
              onCheckReadiness={() => void readiness.refetch()}
            >
              <SemanticReadyNotice
                keywordOnly={
                  searchProfile.keywordSearch === true && !searchProfile.semanticDefault
                }
                ready={
                  searchProfile.semanticDefault !== undefined &&
                  !readiness.data?.saved_writes_paused
                }
                rerun={section === "search" ? workspace.rerunSemantic : undefined}
              >
                <Activity
                  mode={!routeError && section === "search" ? "visible" : "hidden"}
                >
                  <Suspense
                    fallback={
                      <main className="detail-empty">
                        <LoadingStatus label="Loading search…" />
                      </main>
                    }
                  >
                    <SearchView
                      workspace={workspace}
                      historyState={searchReturn?.location.state}
                    />
                  </Suspense>
                </Activity>
                {routeError ?? <Outlet />}
                {active && <SelectionSearch key={location.href} />}
              </SemanticReadyNotice>
            </AppShell>
          </LinkPickerContext>
        </ReferenceHelpHost>
      </SavedWritesPaused>
    </SearchProfileContext>
  );
}
function readStoredColorScheme(): ColorSchemePreference {
  try {
    const value = localStorage.getItem(COLOR_SCHEME_STORAGE_KEY);
    return value === "light" || value === "dark" || value === "system"
      ? value
      : "system";
  } catch {
    return "system";
  }
}
function useSystemColorScheme(): ResolvedColorScheme {
  const [systemColorScheme, setSystemColorScheme] = useState<ResolvedColorScheme>(() =>
    getSystemColorScheme(),
  );
  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => setSystemColorScheme(getSystemColorScheme());
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return systemColorScheme;
}
function getSystemColorScheme(): ResolvedColorScheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}
