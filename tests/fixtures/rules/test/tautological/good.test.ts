import assert from "node:assert";
import { add } from "./math";

describe("math", () => {
  it("adds", () => {
    expect(add(1, 2)).toBe(3);
    expect(1).not.toBe(1);
    expect(1).toBe(2);
    expect("a").toBe('a');
    expect(add(0, 0)).toBeTruthy();
    expect(false).toBeTruthy();
    assert(add(1, 1) === 2);
    assert.equal(add(2, 2), 4);
    assert.strictEqual(-1, 1);
  });
});
