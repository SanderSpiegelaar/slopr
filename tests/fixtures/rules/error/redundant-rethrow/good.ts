declare function parse(text: string): number;

export function wrapped(text: string): number {
  try {
    return parse(text);
  } catch (err) {
    throw new Error(`could not parse ${text}`, { cause: err });
  }
}
