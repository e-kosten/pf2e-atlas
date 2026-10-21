import type { ReactNode } from "react";

/** Shared document chrome; callers supply identity and document-specific actions. */
export function DocumentIdentityHeader({
  title,
  classification,
  navigation,
  identityActions,
  classificationControl,
  hideTitle = false,
  children,
}: {
  title: ReactNode;
  classification: string;
  navigation?: ReactNode;
  identityActions?: ReactNode;
  classificationControl?: ReactNode;
  hideTitle?: boolean;
  children?: ReactNode;
}) {
  return (
    <header className="record-reference__header">
      {navigation && <div className="record-reference__navigation">{navigation}</div>}
      <div className="record-reference__title">
        <div className="record-reference__identity">
          {!hideTitle && <h2>{title}</h2>}
          {identityActions}
        </div>
        <div className="record-reference__classification">
          <p className="record-reference__kind">{classification}</p>
          {classificationControl}
        </div>
      </div>
      {children}
    </header>
  );
}
