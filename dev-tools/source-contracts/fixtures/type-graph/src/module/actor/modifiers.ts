export interface ModifierAdjustment {
  slug: string | null;
  test: (options: string[]) => boolean;
  getNewValue?: (current: number) => number;
  getDamageType?: (current: string | null) => string | null;
  damageType?: string;
  relabel?: string;
  suppress?: boolean;
}
