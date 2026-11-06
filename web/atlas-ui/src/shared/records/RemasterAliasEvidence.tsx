import type { RemasterAliasMatchView } from "../../generated/atlas";

/** A delivered lookup reason retains its human scopes without a source inspector. */
export function RemasterAliasEvidence({
  matches,
}: {
  matches: RemasterAliasMatchView[];
}) {
  if (!matches.length) return null;
  return (
    <div className="remaster-alias-evidence">
      {matches.map((match, index) => {
        const scopes = [
          ...new Set(
            match.lookup_context.supports
              .map((support) => support.context.join(" · "))
              .filter(Boolean),
          ),
        ];
        return (
          <p key={index}>
            {match.lookup_context.label}: “{match.matched_text}”
            {scopes.length > 0 && (
              <span className="remaster-information__context">
                {" "}
                · {scopes.join("; ")}
              </span>
            )}
          </p>
        );
      })}
    </div>
  );
}
