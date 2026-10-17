import { Select } from "antd";
import { useId } from "react";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";

export function SearchModelSelect() {
  const profile = useSearchProfile();
  const controlId = useId();
  const stored = profile.profiles.filter((item) => item.stored_vectors);
  if (profile.loading || stored.length === 1) return null;
  const options = stored.map((item) => ({
    value: item.id,
    label: item.label,
    disabled: !item.stored_vectors,
  }));
  if (profile.id !== null && !options.some((item) => item.value === profile.id))
    options.push({ value: profile.id, label: profile.id, disabled: true });
  return (
    <Select
      id={controlId}
      className="search-model-select"
      aria-label="Search model"
      value={profile.id ?? undefined}
      options={options}
      status={profile.error ? "error" : undefined}
      onChange={profile.select}
      popupMatchSelectWidth={false}
    />
  );
}
