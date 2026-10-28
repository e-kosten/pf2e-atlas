import { HelpTooltip } from "../ui/HelpTooltip";

export function SearchSettingLabel({ label, help }: { label: string; help: string }) {
  return (
    <span className="search-settings__label">
      <span>{label}</span>
      <HelpTooltip label={`About ${label.toLowerCase()}`}>{help}</HelpTooltip>
    </span>
  );
}
