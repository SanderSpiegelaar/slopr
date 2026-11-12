declare const value: unknown;

// slopr-ignore-next-line comment/untracked-todo, ts/unsafe-assertion -- vendor typings lack this field
const hidden = value as any; // TODO: drop the cast
const shown = value as any; // TODO: drop the cast

export { hidden, shown };
