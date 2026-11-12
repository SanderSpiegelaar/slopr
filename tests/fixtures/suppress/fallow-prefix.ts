declare const value: unknown;

// slopr-ignore-next-line fallow/unused-export, ts/unsafe-assertion -- plugins import this
export const hidden = value as any;
export const shown = value as any;
