export function EntityIndexPage({
  actions,
  children,
  className,
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
        <section className="entity-index-page__toolbar">
          <div>
            <h2>{title}</h2>
            {summary ? <p>{summary}</p> : null}
          </div>
          {actions}
        </section>
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
