declare const value: unknown;

const loose = value as any; // expect: ts/unsafe-assertion
const forced = value as unknown as string[]; // expect: ts/unsafe-assertion
const wrapped = value as unknown as number; // expect: ts/unsafe-assertion
const legacy = <any>value; // expect: ts/unsafe-assertion

function read(input: object): string {
  return (input as any).name; // expect: ts/unsafe-assertion
}

export { loose, forced, wrapped, legacy, read };
