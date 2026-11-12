export function parse(input: string): number {
  if (!input) {
    throw new Error("not implemented for empty input");
  }
  return Number(input);
}

export function fail(code: number): never {
  throw new Error(`request failed with ${code}`);
}

export function rethrow(): void {
  throw new TypeError("not implemented");
}

export abstract class Shape {
  abstract area(): number;
}
