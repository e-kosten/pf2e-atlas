export type MetadataGlossaryField = "traits";

export interface MetadataGlossaryEntry {
  value: string;
  label: string;
  description: string | null;
}

export interface MetadataGlossaryArtifact {
  generatedAt: string;
  fields: Partial<Record<MetadataGlossaryField, Record<string, MetadataGlossaryEntry>>>;
}
