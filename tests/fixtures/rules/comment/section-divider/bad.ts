// ======================== // expect: comment/section-divider
import { readFileSync } from "node:fs";

/* --- Setup --- */ // expect: comment/section-divider
export const read = readFileSync;
/*----------*/ // expect: comment/section-divider
