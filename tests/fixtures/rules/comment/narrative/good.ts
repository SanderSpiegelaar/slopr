/** This function adds two numbers. */
export function add(a: number, b: number): number {
  // eslint-disable-next-line no-magic-numbers
  return a + b; // Floats lose precision past 2^53, so callers clamp.
}

// This functionality is shared with the worker.
export const helper = add;
