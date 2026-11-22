import type { ReactNode } from "react";

export function DirectoryHeader({
  title,
  actions,
  summary,
  navigation,
}: {
  title: string;
  actions?: ReactNode;
  summary?: ReactNode;
  navigation?: ReactNode;
}) {
  return (
    <header className="directory-header">
      {navigation}
      <div className="directory-header__heading">
        <div>
          <h1>{title}</h1>
          {summary && <p>{summary}</p>}
        </div>
        {actions}
      </div>
    </header>
  );
}

export function EntityIndexPage({
  actions,
  children,
  className,
  navigation,
  overlays,
  summary,
  title,
  toolbar,
  scrollRef,
  onScroll,
}: {
  actions?: React.ReactNode;
  children: React.ReactNode;
  className: string;
  navigation?: ReactNode;
  overlays?: React.ReactNode;
  summary?: React.ReactNode;
  title: string;
  toolbar?: React.ReactNode;
  scrollRef?: React.Ref<HTMLElement>;
  onScroll?: React.UIEventHandler<HTMLElement>;
}) {
  return (
    <>
      <main
        className={`entity-index-page ${className}${toolbar ? " entity-index-page--with-controls" : ""}`}
      >
        <DirectoryHeader
          title={title}
          actions={actions}
          summary={summary}
          navigation={navigation}
        />
        {toolbar && <div className="entity-index-page__controls">{toolbar}</div>}
        <section
          className="entity-index-page__table"
          ref={scrollRef}
          onScroll={onScroll}
        >
          {children}
        </section>
      </main>
      {overlays}
    </>
  );
}
