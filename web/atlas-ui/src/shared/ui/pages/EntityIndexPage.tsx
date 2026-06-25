export function EntityIndexPage({
  actions,
  children,
  className,
  overlays,
  summary,
  title,
}: {
  actions?: React.ReactNode;
  children: React.ReactNode;
  className: string;
  overlays?: React.ReactNode;
  summary?: React.ReactNode;
  title: string;
}) {
  return (
    <>
      <main className={className}>
        <section className="entity-index-page__toolbar">
          <div>
            <h2>{title}</h2>
            {summary ? <p>{summary}</p> : null}
          </div>
          {actions}
        </section>
        <section className="entity-index-page__table">{children}</section>
      </main>
      {overlays}
    </>
  );
}
