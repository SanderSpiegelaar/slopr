declare const props: unknown;

export function Card() {
  const data = props as any; // expect: ts/unsafe-assertion
  const title = props as unknown as string; // expect: ts/unsafe-assertion
  return <div title={title}>{data.body}</div>;
}
