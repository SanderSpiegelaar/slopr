declare function parse(text: string): number;

export function noisy(text: string): number {
  try {
    return parse(text);
  } catch (err) { // expect: error/redundant-rethrow
    throw err;
  }
}
