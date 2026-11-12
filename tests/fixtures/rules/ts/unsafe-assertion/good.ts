declare const value: unknown;

const sizes = [1, 2, 3] as const;
const name = value as string;
const known = value as unknown;
const scores = { a: 1 } satisfies Record<string, number>;

export { sizes, name, known, scores };
