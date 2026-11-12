import assert from "node:assert";

describe("math", () => {
  it("works", () => {
    expect(true).toBe(true); // expect: test/tautological
    expect(1).toEqual(1); // expect: test/tautological
    expect("a").toStrictEqual("a"); // expect: test/tautological
    expect(null).toBe(null); // expect: test/tautological
    expect(true).toBeTruthy(); // expect: test/tautological
    assert(true); // expect: test/tautological
    assert.ok(true); // expect: test/tautological
    assert.equal(2, 2); // expect: test/tautological
    assert.strictEqual("x", "x", "same"); // expect: test/tautological
  });
});
