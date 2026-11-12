declare const value: unknown;

const first = value as any;
const second = <any>value;
// TODO: drop the casts
export { first, second };

// slopr-ignore-file ts/unsafe-assertion -- generated client
