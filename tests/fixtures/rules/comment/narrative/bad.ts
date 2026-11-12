// This function adds two numbers // expect: comment/narrative
export function add(a: number, b: number): number {
  // It returns the sum // expect: comment/narrative
  return a + b;
}

/* Then we export the helper */ // expect: comment/narrative
export const helper = add;
