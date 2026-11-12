// slopr-ignore-file suppress/invalid-directive -- r
declare const value: unknown;

// slopr-ignore-next-line ts/unsafe-assertion -- vendor typings lack this field
const hidden = value as any;
// slopr-ignore-next-lien ts/unsafe-assertion -- vendor typings lack this field
const shown = value as any;

export { hidden, shown };
