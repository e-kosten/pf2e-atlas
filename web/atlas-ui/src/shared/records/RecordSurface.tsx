import { Popover } from "antd";
import type React from "react";
import type {
  RecordDetailView,
  RecordSurfaceSectionKindView,
  RecordSurfaceSectionView,
  RecordSurfaceView,
  SurfaceActivityView,
  SurfaceAdjustmentView,
  SurfaceNoteView,
  SurfaceScalarView,
  SurfaceValueView,
} from "../../generated/atlas";
import { PresentationContentView, RecordPresentation } from "./RecordPresentation";

type RecordSurfaceProps = {
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
  surface: RecordSurfaceView;
  slots?: Record<string, React.ReactNode>;
};

export function RecordSurface({
  onReference,
  surface,
  slots = {},
}: RecordSurfaceProps) {
  if (surface.profile === "search_compact") {
    return <SearchCompactSurface onReference={onReference} surface={surface} />;
  }
  if (surface.profile === "encounter_participant") {
    return (
      <EncounterParticipantSurface
        onReference={onReference}
        slots={slots}
        surface={surface}
      />
    );
  }
  return (
    <GenericRecordSurface onReference={onReference} slots={slots} surface={surface} />
  );
}

function GenericRecordSurface({
  onReference,
  surface,
  slots = {},
}: RecordSurfaceProps) {
  return (
    <article
      className={[
        "record-surface",
        `record-surface--${surface.profile.replace(/_/g, "-")}`,
      ].join(" ")}
    >
      <header className="record-surface__header">
        <div>
          <p className="eyebrow">{surface.header.kind_label ?? surface.kind}</p>
          <h2>{surface.title}</h2>
          {surface.profile !== "search_compact" && (
            <p className="record-key">{surface.record_key}</p>
          )}
        </div>
        <div className="record-surface__header-meta">
          {surface.header.level_label && (
            <span className="record-surface__level">
              Level {surface.header.level_label}
            </span>
          )}
          <div className="badge-row">
            {surface.header.traits.map((trait) => (
              <span className="atlas-badge" key={`${trait.kind}-${trait.value}`}>
                {trait.label}
              </span>
            ))}
          </div>
        </div>
      </header>
      {slots.header}
      <div className="record-surface__sections">
        {surface.sections.map((section) => (
          <SurfaceSection
            key={`${section.kind}-${section.title}`}
            onReference={onReference}
            section={section}
            slot={slots[section.kind]}
          />
        ))}
      </div>
      {surface.fallback_presentation && surface.profile !== "search_compact" && (
        <details
          className="record-surface__fallback"
          open={surface.profile === "record_detail"}
        >
          <summary>Source presentation</summary>
          <RecordPresentation
            detail={fallbackDetail(surface)}
            loading={false}
            onReference={onReference}
          />
        </details>
      )}
    </article>
  );
}

function SearchCompactSurface({
  onReference,
  surface,
}: {
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
  surface: RecordSurfaceView;
}) {
  const description = sectionByKind(surface, "description");
  const facts = sectionByKind(surface, "identity");
  return (
    <article className="record-surface record-surface--search-compact">
      <div className="record-surface-search__identity">
        <p className="eyebrow">{surface.header.kind_label ?? surface.kind}</p>
        <h2>{surface.title}</h2>
        {description?.content && (
          <div className="record-surface-search__description">
            <PresentationContentView
              content={description.content}
              onReference={onReference}
            />
          </div>
        )}
        {surface.header.traits.length > 0 && (
          <div className="record-surface-search__traits">
            {surface.header.traits.map((trait) => (
              <span className="atlas-badge" key={`${trait.kind}-${trait.value}`}>
                {trait.label}
              </span>
            ))}
          </div>
        )}
      </div>
      {facts && (
        <div className="record-surface-search__facts">
          <CompactFactRows values={facts.values ?? []} />
        </div>
      )}
      <div className="record-surface-search__meta">
        {surface.header.level_label && <span>Level {surface.header.level_label}</span>}
        {(surface.header.publication ?? surface.header.pack) && (
          <small>{surface.header.publication ?? surface.header.pack}</small>
        )}
      </div>
    </article>
  );
}

function EncounterParticipantSurface({
  onReference,
  slots = {},
  surface,
}: RecordSurfaceProps) {
  const runtime = sectionByKind(surface, "runtime");
  const vitals = sectionByKind(surface, "vitals");
  const defenses = sectionByKind(surface, "defenses");
  const saves = sectionByKind(surface, "saves");
  const abilities = sectionByKind(surface, "abilities");
  const skills = sectionByKind(surface, "skills");
  const movement = sectionByKind(surface, "movement");
  const activities = sectionByKind(surface, "activities");
  const description = sectionByKind(surface, "description");
  const references = sectionByKind(surface, "references");

  return (
    <article className="record-surface record-surface--encounter-participant">
      <header className="record-surface-structured__header">
        <div className="record-surface-structured__identity">
          <p className="eyebrow">{surface.header.kind_label ?? surface.kind}</p>
          <h2>{surface.title}</h2>
          <div className="badge-row">
            {surface.header.traits.map((trait) => (
              <span className="atlas-badge" key={`${trait.kind}-${trait.value}`}>
                {trait.label}
              </span>
            ))}
          </div>
        </div>
        <div className="record-surface-structured__header-actions">
          {slots.header_actions}
        </div>
      </header>

      {slots.header && (
        <div className="record-surface-structured__participant">{slots.header}</div>
      )}

      <div className="record-surface-structured__runtime">
        <SurfaceCard className="record-surface-card--vitals" title="Vitals">
          {slots.vitals ?? (vitals && <CompactFactRows values={vitals.values ?? []} />)}
        </SurfaceCard>
        <SurfaceCard className="record-surface-card--conditions" title="Conditions">
          {slots.conditions}
        </SurfaceCard>
      </div>

      <div className="record-surface-structured__grid">
        <div className="record-surface-structured__column">
          {defenses && (
            <SurfaceCard title="Defenses">
              <CompactFactRows values={defenses.values ?? []} />
            </SurfaceCard>
          )}
          {saves && (
            <SurfaceCard title="Saves">
              <CompactFactRows values={saves.values ?? []} />
            </SurfaceCard>
          )}
          {runtime && (
            <SurfaceCard title="Runtime">
              <CompactFactRows values={runtime.values ?? []} />
              <SurfaceNotes notes={runtime.notes ?? []} />
            </SurfaceCard>
          )}
          {movement && (
            <SurfaceCard title="Senses & Movement">
              <CompactFactRows values={movement.values ?? []} />
              <SurfaceNotes notes={movement.notes ?? []} />
            </SurfaceCard>
          )}
          {abilities && (
            <SurfaceCard title="Abilities">
              <CompactFactRows columns={2} values={abilities.values ?? []} />
            </SurfaceCard>
          )}
          {skills && (
            <SurfaceCard title="Skills">
              <CompactFactRows columns={2} values={skills.values ?? []} />
            </SurfaceCard>
          )}
          {references && (
            <SurfaceCard title={references.title}>
              <SurfaceNotes notes={references.notes ?? []} />
            </SurfaceCard>
          )}
        </div>

        <div className="record-surface-structured__column">
          {activities && (
            <SurfaceCard className="record-surface-card--activities" title="Activities">
              <div className="record-surface-structured__activities">
                {(activities.activities ?? []).map((activity) => (
                  <SurfaceActivity activity={activity} key={activity.key} />
                ))}
              </div>
            </SurfaceCard>
          )}
          {description?.content && (
            <SurfaceCard title="Description">
              <details className="record-surface__description-details">
                <summary>Description</summary>
                <div className="record-surface__content">
                  <PresentationContentView
                    content={description.content}
                    onReference={onReference}
                  />
                </div>
              </details>
            </SurfaceCard>
          )}
          {slots.notes && (
            <SurfaceCard title="Participant Note">{slots.notes}</SurfaceCard>
          )}
        </div>
      </div>

      {surface.fallback_presentation && (
        <details className="record-surface__fallback">
          <summary>Source presentation</summary>
          <RecordPresentation
            detail={fallbackDetail(surface)}
            loading={false}
            onReference={onReference}
          />
        </details>
      )}
    </article>
  );
}

function SurfaceCard({
  children,
  className = "",
  title,
}: {
  children: React.ReactNode;
  className?: string;
  title: string;
}) {
  return (
    <section className={["record-surface-card", className].filter(Boolean).join(" ")}>
      <h3>{title}</h3>
      {children}
    </section>
  );
}

function CompactFactRows({
  columns = 1,
  values,
}: {
  columns?: 1 | 2;
  values: SurfaceValueView[];
}) {
  if (values.length === 0) {
    return null;
  }
  return (
    <dl
      className={[
        "record-surface-fact-rows",
        columns === 2 ? "record-surface-fact-rows--two" : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {values.map((value) => (
        <div key={value.key}>
          <dt>{value.label}</dt>
          <dd>
            <SurfaceValue value={value} />
          </dd>
        </div>
      ))}
    </dl>
  );
}

function sectionByKind(surface: RecordSurfaceView, kind: RecordSurfaceSectionKindView) {
  return surface.sections.find((section) => section.kind === kind);
}

function SurfaceSection({
  onReference,
  section,
  slot,
}: {
  onReference: (recordKey: string, anchorRect?: DOMRect) => void;
  section: RecordSurfaceSectionView;
  slot: React.ReactNode;
}) {
  const hasSlot = slot !== undefined && slot !== null;
  const body = (
    <>
      {slot}
      {!hasSlot && section.content && (
        <div className="record-surface__content">
          <PresentationContentView
            content={section.content}
            onReference={onReference}
          />
        </div>
      )}
      {!hasSlot && section.values && section.values.length > 0 && (
        <div className="record-surface__values">
          {section.values.map((value) => (
            <SurfaceValue key={value.key} value={value} />
          ))}
        </div>
      )}
      {!hasSlot && section.groups && section.groups.length > 0 && (
        <div className="record-surface__groups">
          {section.groups.map((group) => (
            <div className="record-surface__group" key={group.key}>
              <h4>{group.label}</h4>
              <div className="record-surface__values">
                {group.values.map((value) => (
                  <SurfaceValue key={value.key} value={value} />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
      {!hasSlot && section.activities && section.activities.length > 0 && (
        <div className="record-surface__activities">
          {section.activities.map((activity) => (
            <SurfaceActivity activity={activity} key={activity.key} />
          ))}
        </div>
      )}
      {!hasSlot && section.notes && section.notes.length > 0 && (
        <SurfaceNotes notes={section.notes} />
      )}
    </>
  );
  if (section.collapsed_by_default) {
    return (
      <details className="record-surface__section record-surface__section--collapsible">
        <summary>{section.title}</summary>
        {body}
      </details>
    );
  }
  return (
    <section className="record-surface__section">
      <h3>{section.title}</h3>
      {body}
    </section>
  );
}

function SurfaceActivity({ activity }: { activity: SurfaceActivityView }) {
  return (
    <section className="record-surface__activity">
      <header>
        <strong>{activity.label}</strong>
        <span>{activity.kind}</span>
      </header>
      {(activity.values?.length ?? 0) > 0 && (
        <div className="record-surface__activity-values">
          {activity.values?.map((value) => (
            <SurfaceValue key={value.key} value={value} />
          ))}
        </div>
      )}
      {activity.groups?.map((group) => (
        <div className="record-surface__activity-mode" key={group.key}>
          <h4>{group.label}</h4>
          <div className="record-surface__activity-values">
            {group.values?.map((value) => (
              <SurfaceValue key={value.key} value={value} />
            ))}
          </div>
        </div>
      ))}
    </section>
  );
}

function SurfaceValue({ value }: { value: SurfaceValueView }) {
  const className = [
    "record-surface-value",
    value.adjusted ? "record-surface-value--adjusted" : "",
    hasSurfaceValueExplanation(value) ? "record-surface-value--explainable" : "",
  ]
    .filter(Boolean)
    .join(" ");
  if (!hasSurfaceValueExplanation(value)) {
    return (
      <div className={className}>
        <span>{value.label}</span>
        <strong>{formatScalar(value.value, value.display)}</strong>
      </div>
    );
  }
  return (
    <Popover
      content={
        <SurfaceValueExplanation
          adjustments={value.adjustments ?? []}
          baseValue={value.base_value}
          display={value.display}
          finalValue={value.value}
          notes={value.notes ?? []}
          suppressed={value.suppressed_adjustments ?? []}
        />
      }
      placement="top"
      trigger="click"
    >
      <button
        aria-label={`Show explanation for ${value.label}`}
        className={className}
        type="button"
      >
        <span>{value.label}</span>
        <strong>{formatScalar(value.value, value.display)}</strong>
      </button>
    </Popover>
  );
}

function hasSurfaceValueExplanation(value: SurfaceValueView) {
  return (
    value.adjusted ||
    (value.adjustments?.length ?? 0) > 0 ||
    (value.suppressed_adjustments?.length ?? 0) > 0 ||
    (value.notes?.length ?? 0) > 0
  );
}

function SurfaceValueExplanation({
  adjustments,
  baseValue,
  display,
  finalValue,
  notes,
  suppressed,
}: {
  adjustments: SurfaceAdjustmentView[];
  baseValue: SurfaceScalarView | undefined;
  display: SurfaceValueView["display"];
  finalValue: SurfaceScalarView;
  notes: SurfaceNoteView[];
  suppressed: SurfaceAdjustmentView[];
}) {
  return (
    <div className="record-surface-explanation">
      {baseValue && (
        <p>
          Base: {formatScalar(baseValue, display)}
          <br />
          Final: {formatScalar(finalValue, display)}
        </p>
      )}
      {adjustments.map((adjustment) => (
        <p key={`${adjustment.source}-${adjustment.label}`}>
          {adjustment.label}
          {adjustment.delta ? ` ${formatAdjustment(adjustment.delta)}` : ""}
          {adjustment.reason ? ` (${adjustment.reason})` : ""}
        </p>
      ))}
      {suppressed.map((adjustment) => (
        <p key={`suppressed-${adjustment.source}-${adjustment.label}`}>
          Suppressed: {adjustment.label}
          {adjustment.delta ? ` ${formatAdjustment(adjustment.delta)}` : ""}
        </p>
      ))}
      {notes.map((note) => (
        <p key={`${note.source ?? ""}-${note.label}`}>{note.text}</p>
      ))}
    </div>
  );
}

function SurfaceNotes({ notes }: { notes: SurfaceNoteView[] }) {
  return (
    <div className="record-surface__notes">
      {notes.map((note) => (
        <p key={`${note.source ?? ""}-${note.label}`}>
          <strong>{note.label}</strong>: {note.text}
        </p>
      ))}
    </div>
  );
}

function formatScalar(
  scalar: SurfaceScalarView,
  display: SurfaceValueView["display"],
): string {
  switch (scalar.kind) {
    case "number":
      return display === "signed_modifier"
        ? formatSigned(Number(scalar.value))
        : Number(scalar.value).toString();
    case "distance_feet":
      return `${Number(scalar.value).toString()} ft`;
    case "formula":
    case "text":
      return scalar.value;
  }
}

function formatAdjustment(scalar: SurfaceScalarView): string {
  if (scalar.kind === "number") {
    return formatSigned(Number(scalar.value));
  }
  if (scalar.kind === "distance_feet") {
    return `${formatSigned(Number(scalar.value))} ft`;
  }
  return scalar.value;
}

function formatSigned(value: number): string {
  if (value > 0) {
    return `+${value.toString()}`;
  }
  return value.toString();
}

function fallbackDetail(surface: RecordSurfaceView): RecordDetailView {
  return {
    record_key: surface.record_key,
    title: surface.title,
    kind: surface.kind,
    presentation: surface.fallback_presentation!,
    surface: undefined,
  };
}
