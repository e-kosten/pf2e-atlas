import { createContext, useContext, Fragment, type ReactNode } from "react";
import { AtlasLink } from "../../app/AtlasLink";
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
) => void;
const HelpContext = createContext<{
  onHelp?: ReferenceHelpHandler;
  generation?: string;
}>({});

export function ReferenceHelpProvider({
  onHelp,
  generation,
  children,
}: {
  onHelp?: ReferenceHelpHandler;
  generation?: string;
  children: ReactNode;
}) {
  const inherited = useContext(HelpContext);
  return (
    <HelpContext
      value={{
        onHelp: onHelp ?? inherited.onHelp,
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
        readingContext: destination.reading_context,
        definitionKey: destination.reading_context
          ? (definitionKey ?? undefined)
          : undefined,
      });
}

export function useReferenceHelpNavigation() {
  const host = useContext(HelpContext);
  return (help: ReferenceHelpView, anchor: HTMLElement, generation?: string) => {
    if (!help.destination) return;
    if (host.onHelp) host.onHelp(help, anchor, generation ?? host.generation);
    else
      navigateDestination(referenceDestination(help.destination, help.definition_key));
  };
}

/** Native links and local reading carry the same complete supplied purpose. */
export function ReferenceHelpLink({
  help,
  children,
  generation,
  className,
}: {
  help: ReferenceHelpView | null | undefined;
  children: ReactNode;
  generation?: string;
  className?: string;
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
        open(help, event.currentTarget, generation);
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
