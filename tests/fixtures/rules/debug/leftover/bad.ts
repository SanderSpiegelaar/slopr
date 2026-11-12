export function total(items: number[]): number {
  console.log(items); // expect: debug/leftover
  console.debug("sum"); // expect: debug/leftover
  console.info("sum"); // expect: debug/leftover
  console.trace(); // expect: debug/leftover
  console.dir(items); // expect: debug/leftover
  debugger; // expect: debug/leftover
  console.error("kept");
  console.warn("kept");
  return items.reduce((a, b) => a + b, 0);
}
