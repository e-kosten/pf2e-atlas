import type { ReactNode, Ref } from "react";

/** Shared document chrome; callers supply identity and document-specific actions. */
export function DocumentIdentityHeader({
  title,
  classification,
  navigation,
  identityActions,
  classificationControl,
  stacked = false,
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
  const kind =
    classification || classificationControl ? (
      <div className="record-reference__classification">
        {classification && <p className="record-reference__kind">{classification}</p>}
        {classificationControl}
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
        {!stacked && kind}
        {actions && <div className="record-reference__actions">{actions}</div>}
      </div>
      {stacked ? (
        <div className="record-reference__metadata">
          {kind}
          {children}
        </div>
      ) : (
        children
      )}
      {attribution}
    </header>
  );
}
