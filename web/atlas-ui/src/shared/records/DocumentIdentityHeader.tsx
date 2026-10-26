import type { ReactNode, Ref } from "react";

/** Shared document chrome; callers supply identity and document-specific actions. */
export function DocumentIdentityHeader({
  title,
  classification,
  navigation,
  identityActions,
  classificationControl,
  stacked = true,
  headingRef,
  headingId,
  actions,
  attribution,
  children,
}: {
  title: ReactNode;
  classification: string;
  navigation?: ReactNode;
  identityActions?: ReactNode;
  classificationControl?: ReactNode;
  stacked?: boolean;
  headingRef?: Ref<HTMLHeadingElement>;
  headingId?: string;
  actions?: ReactNode;
  attribution?: ReactNode;
  children?: ReactNode;
}) {
  const kind = classification ? (
    <div className="record-reference__classification">
      {classificationControl ?? (
        <p className="record-reference__kind">{classification}</p>
      )}
    </div>
  ) : null;
  return (
    <header
      className={`record-reference__header${stacked ? " record-reference__header--stacked" : ""}`}
    >
      {navigation && <div className="record-reference__navigation">{navigation}</div>}
      <div className="record-reference__title">
        <div className="record-reference__identity">
          <h2 id={headingId} ref={headingRef} tabIndex={headingRef ? -1 : undefined}>
            {title}
          </h2>
          {identityActions}
        </div>
        {(kind || actions) && (
          <div className="record-reference__tools">
            {kind}
            {actions && <div className="record-reference__actions">{actions}</div>}
          </div>
        )}
      </div>
      {stacked && children ? (
        <div className="record-reference__metadata">{children}</div>
      ) : (
        !stacked && children
      )}
      {attribution}
    </header>
  );
}
