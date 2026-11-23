import { Typography } from "antd";
import { AtlasApiError } from "../../api/apiError";
import { listDestination } from "../../app/destinations";
import { destinationHref } from "../../app/navigation";
import { errorExplanation } from "../../shared/feedback/errorMessages";

export function listNameConflictHelp(error: unknown) {
  if (
    !(error instanceof AtlasApiError) ||
    error.appError?.code !== "saved_list_name_conflict"
  )
    return undefined;
  const details = error.appError.details;
  if (
    !details ||
    typeof details !== "object" ||
    !("slug" in details) ||
    typeof details.slug !== "string" ||
    !("name" in details) ||
    typeof details.name !== "string" ||
    !("archived" in details) ||
    typeof details.archived !== "boolean"
  )
    return errorExplanation(error);
  return (
    <span role="alert">
      {details.archived ? "An archived list" : "A list"} named “
      <Typography.Link
        href={destinationHref(
          listDestination({ slug: details.slug, selectedRecordKey: null }),
        )}
        target="_blank"
        rel="noopener"
      >
        {details.name}
      </Typography.Link>
      ” already exists.
    </span>
  );
}
