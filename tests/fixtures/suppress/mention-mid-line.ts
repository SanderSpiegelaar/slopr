declare const value: unknown;

// slopr-ignore-next-line ts/unsafe-assertion -- vendor typings lack this field
const hidden = value as any;
// see slopr-ignore-next-line ts/unsafe-assertion -- vendor typings lack this field
const shown = value as any;

export { hidden, shown };
