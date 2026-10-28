import { Button } from "antd";
import { Search } from "lucide-react";
import type { ReactNode } from "react";
import { searchDestination } from "../../app/destinations";
import {
  destinationHref,
  isPrimaryActivation,
  navigateDestination,
} from "../../app/navigation";

/** A preview's main action continues its scope in the full search workspace. */
export function ExploreSearchAction({
  route,
  children,
  summary,
  disabled = false,
}: {
  route: ReturnType<typeof searchDestination>;
  children: ReactNode;
  summary?: ReactNode;
  disabled?: boolean;
}) {
  return (
    <div className="record-explore__search-toolbar">
      {summary && <span className="record-explore__search-summary">{summary}</span>}
      <Button
        type="default"
        disabled={disabled}
        icon={<Search size={16} />}
        href={destinationHref(route)}
        onClick={(event) => {
          if (!isPrimaryActivation(event)) return;
          event.preventDefault();
          navigateDestination(route);
        }}
      >
        {children}
      </Button>
    </div>
  );
}
