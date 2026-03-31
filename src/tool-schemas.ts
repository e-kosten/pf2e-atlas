import * as z from "zod/v4";

import { SEARCH_CATEGORIES } from "./categories.js";

export const searchCategorySchema = z.enum(SEARCH_CATEGORIES);
export const searchModeSchema = z.enum(["structured", "lexical", "hybrid"]);
export const listRecordsModeSchema = z.literal("structured");
