declare const value: unknown;

// slopr-ignore-next-line ts/unsafe-assertion -- vendor typings lack this field
const hidden = value as any;
// slopr-ignore-next-line ts/unsafe-assertion
const shown = value as any;
// slopr-ignore-next-line ts/unsafe-assertion --
const alsoShown = value as any;

export { hidden, shown, alsoShown };
