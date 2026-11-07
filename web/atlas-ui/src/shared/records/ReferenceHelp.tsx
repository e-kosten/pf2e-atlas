import { createContext, useContext, Fragment, type ReactNode } from "react";
import { AtlasLink } from "../../app/AtlasLink";
import type { RecordReferenceHandler } from "./PreparedContent";
import type {
  RecordNavigationView,
  ReferenceDestinationView,
  ReferenceHelpView,
  ReferenceValueView,
} from "../../generated/atlas";
import { definitionDestination, recordDestination } from "../../app/destinations";
import { isPrimaryActivation, navigateDestination } from "../../app/navigation";

export type ReferenceHelpHandler = (
  help: ReferenceHelpView,
  anchor: HTMLElement,
  generation?: string,
  returnToOwner?: ReferenceHelpReturn,
) => void;
export type ReferenceHelpCloseReason = "return" | "navigate";
export type ReferenceHelpReturn = (reason: ReferenceHelpCloseReason) => void;
export type ReferenceHelpHandoff = (
  anchor: HTMLElement,
  start: (finish: ReferenceHelpReturn) => void,
) => void;
const HelpContext = createContext<{
  onHelp?: ReferenceHelpHandler;
  onReference?: RecordReferenceHandler;
  generation?: string;
}>({});

export function ReferenceHelpProvider({
  onHelp,
  onReference,
  generation,
  children,
}: {
  onHelp?: ReferenceHelpHandler;
  onReference?: RecordReferenceHandler;
  generation?: string;
  children: ReactNode;
}) {
  const inherited = useContext(HelpContext);
  return (
    <HelpContext
      value={{
        onHelp: onHelp ?? inherited.onHelp,
        onReference: onReference ?? (onHelp ? undefined : inherited.onReference),
        generation: generation ?? inherited.generation,
      }}
    >
      {children}
    </HelpContext>
  );
}

export function referenceDestination(
  destination: ReferenceDestinationView,
  definitionKey?: string | null,
  landing?: RecordNavigationView,
) {
  return destination.kind === "definition"
    ? definitionDestination({ definitionKey: destination.key })
    : recordDestination({
        recordKey: destination.navigation.record_key,
        selection: destination.navigation,
        landing,
        definitionKey: definitionKey ?? undefined,
      });
}

/** Classification follows the exact typed identity namespace, never body text. */
export function definitionClassification(key: string): "Trait" | "Definition" {
  return key.startsWith("trait:") ? "Trait" : "Definition";
}

export function useReferenceHelpHandler() {
  return useContext(HelpContext).onHelp;
}

export function useReferenceRecordNavigation() {
  return useContext(HelpContext).onReference;
}

export function useReferenceHelpNavigation() {
  const host = useContext(HelpContext);
  return (
    help: ReferenceHelpView,
    anchor: HTMLElement,
    generation?: string,
    returnToOwner?: ReferenceHelpReturn,
  ) => {
    if (!help.destination) return;
    if (host.onHelp) {
      if (returnToOwner)
        host.onHelp(help, anchor, generation ?? host.generation, returnToOwner);
      else host.onHelp(help, anchor, generation ?? host.generation);
    } else
      navigateDestination(referenceDestination(help.destination, help.definition_key));
  };
}

/** Native links and local reading carry the same complete supplied destination. */
export function ReferenceHelpLink({
  help,
  children,
  generation,
  className,
  handoff,
}: {
  help: ReferenceHelpView | null | undefined;
  children: ReactNode;
  generation?: string;
  className?: string;
  handoff?: ReferenceHelpHandoff;
}) {
  const open = useReferenceHelpNavigation();
  if (!help?.destination) return <>{children}</>;
  const route = referenceDestination(help.destination, help.definition_key);
  return (
    <AtlasLink
      {...route}
      className={className}
      onClick={(event) => {
        event.stopPropagation();
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        const anchor = event.currentTarget;
        if (handoff)
          handoff(anchor, (finish) => open(help, anchor, generation, finish));
        else open(help, anchor, generation);
      }}
    >
      {children}
    </AtlasLink>
  );
}

export function ReferenceValue({ value }: { value: ReferenceValueView }) {
  return <ReferenceHelpLink help={value.help}>{value.label}</ReferenceHelpLink>;
}

export function ReferenceValues({
  values,
  separator = ", ",
}: {
  values: ReferenceValueView[];
  separator?: ReactNode;
}) {
  return (
    <>
      {values.map((value, index) => (
        <Fragment key={JSON.stringify([value.value, index])}>
          {index > 0 && separator}
          <ReferenceValue value={value} />
        </Fragment>
      ))}
    </>
  );
}
