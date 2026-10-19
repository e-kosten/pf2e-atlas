import { ConfigProvider, Alert, App } from "antd";
import { Suspense, lazy, useEffect, useState } from "react";
import {
  COLOR_SCHEME_STORAGE_KEY,
  antDesignTheme,
  atlasCssVariables,
  type ColorSchemePreference,
  type ResolvedColorScheme,
} from "../shared/theme/atlasTheme";
import { AppShell } from "./AppShell";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  currentAtlasRoute,
  atlasRoutePath,
  navigateToAtlasRoute,
  type AtlasRoute,
} from "./routes";
import { useSearchWorkspace } from "../features/search/useSearchWorkspace";
import { useQuery } from "@tanstack/react-query";
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
const EncounterIndexView = lazy(() =>
  import("../features/encounters/EncounterIndexView").then((module) => ({
    default: module.EncounterIndexView,
  })),
);
const EncounterDetailView = lazy(() =>
  import("../features/encounters/EncounterDetailView").then((module) => ({
    default: module.EncounterDetailView,
  })),
);
const EncounterEditView = lazy(() =>
  import("../features/encounters/EncounterEditView").then((module) => ({
    default: module.EncounterEditView,
  })),
);
const ListIndexView = lazy(() =>
  import("../features/lists/ListIndexView").then((module) => ({
    default: module.ListIndexView,
  })),
);
const ListDetailView = lazy(() =>
  import("../features/lists/ListDetailView").then((module) => ({
    default: module.ListDetailView,
  })),
);
const ListEditView = lazy(() =>
  import("../features/lists/ListEditView").then((module) => ({
    default: module.ListEditView,
  })),
);
const RecordView = lazy(() =>
  import("../features/records/RecordViews").then((module) => ({
    default: module.RecordView,
  })),
);
const ReaderView = lazy(() =>
  import("../features/records/RecordViews").then((module) => ({
    default: module.ReaderView,
  })),
);
export function AtlasApp() {
  const [colorScheme, setColorScheme] =
    useState<ColorSchemePreference>(readStoredColorScheme);
  const [route, setRoute] = useState<AtlasRoute>(currentAtlasRoute);
  const systemColorScheme = useSystemColorScheme();
  const resolvedColorScheme =
    colorScheme === "system" ? systemColorScheme : colorScheme;
  const readiness = useQuery({ queryKey: ["readiness"], queryFn: getReadiness });
  const searchProfile = useSearchProfileSelection(readiness.data);
  const workspace = useSearchWorkspace({
    enabled: route.kind === "search",
    searchProfile: searchProfile.id,
    pendingUrlProfile: searchProfile.pendingUrlProfile,
    searchProfileStored: searchProfile.storedVectors,
    searchProfileUsable: searchProfile.usable,
    includeSearchProfileInUrl:
      searchProfile.profiles.filter((profile) => profile.stored_vectors).length !== 1 ||
      !searchProfile.storedVectors,
  });

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedColorScheme;
    Object.entries(atlasCssVariables(resolvedColorScheme)).forEach(
      ([property, value]) => {
        document.documentElement.style.setProperty(property, String(value));
      },
    );
  }, [resolvedColorScheme]);

  useEffect(() => {
    const onRouteChange = () => setRoute(currentAtlasRoute());
    window.addEventListener("popstate", onRouteChange);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    return () => {
      window.removeEventListener("popstate", onRouteChange);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onRouteChange);
    };
  }, []);

  const updateColorScheme = (preference: ColorSchemePreference) => {
    setColorScheme(preference);
    try {
      localStorage.setItem(COLOR_SCHEME_STORAGE_KEY, preference);
    } catch {
      // Browser preferences are optional, including when storage is unavailable.
    }
  };

  return (
    <ConfigProvider
      theme={antDesignTheme(resolvedColorScheme)}
      select={{
        classNames: {
          popup: { root: "atlas-select-popup", listItem: "atlas-select-option" },
        },
      }}
    >
      <App component={false}>
        <SearchProfileContext value={searchProfile}>
          <AppShell
            activeView={route.kind}
            colorScheme={colorScheme}
            onColorSchemeChange={updateColorScheme}
            onNavigateSearch={() =>
              navigateToAtlasRoute({ kind: "search", selectedRecordKey: null })
            }
            onNavigateEncounters={() => navigateToAtlasRoute({ kind: "encounters" })}
            onNavigateLists={() => navigateToAtlasRoute({ kind: "lists" })}
            resolvedColorScheme={resolvedColorScheme}
            readiness={readiness.data}
            readinessError={readiness.error}
            checkingReadiness={readiness.isFetching}
            onCheckReadiness={() => void readiness.refetch()}
          >
            <Suspense fallback={<RouteLoading />}>
              {route.kind === "invalid" && <Alert type="error" title={route.message} />}
              {route.kind === "search" && <SearchView workspace={workspace} />}
              {route.kind === "encounters" && <EncounterIndexView route={route} />}
              {route.kind === "encounter" && <EncounterDetailView route={route} />}
              {route.kind === "encounterEdit" && <EncounterEditView route={route} />}
              {route.kind === "lists" && <ListIndexView route={route} />}
              {route.kind === "list" && (
                <ListDetailView key={route.slug} route={route} />
              )}
              {route.kind === "listEdit" && <ListEditView route={route} />}
              {route.kind === "record" && (
                <RecordView key={atlasRoutePath(route)} route={route} />
              )}
              {route.kind === "reader" && <ReaderView route={route} />}
            </Suspense>
          </AppShell>
        </SearchProfileContext>
      </App>
    </ConfigProvider>
  );
}

function RouteLoading() {
  return <main className="detail-empty">Loading</main>;
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
