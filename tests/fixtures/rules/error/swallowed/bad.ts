declare function load(): Promise<string>;
declare function parse(text: string): number;

export function quiet(text: string): number {
  try {
    return parse(text);
  } catch (err) {} // expect: error/swallowed
  try {
    return parse(text);
  } catch { // expect: error/swallowed
    // ignored
  }
  try {
    return parse(text);
  } catch (err) { // expect: error/swallowed
    console.error("parse failed");
  }
  return 0;
}

load().catch(() => {}); // expect: error/swallowed
load().catch(function () {}); // expect: error/swallowed
