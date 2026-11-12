declare function load(): Promise<string>;
declare function parse(text: string): number;
declare const logger: { error(...args: unknown[]): void };

export function loud(text: string): number {
  try {
    return parse(text);
  } catch (err) {
    console.error("parse failed", err);
  }
  try {
    return parse(text);
  } catch (err) {
    logger.error(`parse failed: ${err}`);
  }
  try {
    return parse(text);
  } catch (err) {
    return -1;
  }
}

load().catch((err) => console.error(err));
load().then(() => {});
