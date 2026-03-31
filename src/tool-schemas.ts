import * as z from "zod/v4";

import { SEARCH_CATEGORIES } from "./categories.js";

export const searchCategorySchema = z.enum(SEARCH_CATEGORIES);
export const searchProfileSchema = z.enum(["lookup", "balanced", "concept"]);
export const sourceCategorySchema = z.enum(["core", "rules", "adventure", "unknown"]);
export const spellKindSchema = z.enum(["focus", "ritual", "cantrip"]);
