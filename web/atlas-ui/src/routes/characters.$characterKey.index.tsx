import { createFileRoute } from "@tanstack/react-router";
import { characterDestination } from "../app/destinations";
import { useAtlasRouter } from "../app/router";
import {
  characterAbilityView,
  type CharacterAbilityView,
  routeSearch,
} from "../app/routeSearch";
import { CharacterPage } from "../features/parties/CharacterPage";

export const Route = createFileRoute("/characters/$characterKey/")({
  ...routeSearch("character"),
  staticData: { section: "encounters" },
  component: Page,
});
function Page() {
  const { characterKey } = Route.useParams();
  const view = Route.useSearch({ select: characterAbilityView });
  const router = useAtlasRouter();
  const base = characterDestination(characterKey);
  const baseHref = router.buildLocation(base).href;
  const navigation = {
    view,
    open(next: CharacterAbilityView) {
      const current = router.latestLocation;
      const replacing = Boolean(characterAbilityView(current.search));
      void router.navigate({
        ...characterDestination(characterKey, next),
        replace: replacing,
        resetScroll: false,
        state: {
          atlasCharacterOrigin: replacing
            ? current.state.atlasCharacterOrigin
            : { href: baseHref, index: current.state.__TSR_index },
        },
      });
    },
    close() {
      const { state } = router.latestLocation;
      const origin = state.atlasCharacterOrigin;
      if (
        origin?.href === baseHref &&
        origin.index + 1 === state.__TSR_index &&
        router.history.canGoBack()
      ) {
        router.history.back();
      } else {
        void router.navigate({ ...base, replace: true, resetScroll: false });
      }
    },
  };
  return <CharacterPage characterKey={characterKey} navigation={navigation} />;
}
