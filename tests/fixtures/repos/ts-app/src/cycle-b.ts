import { a } from "./cycle-a";

export function b(): number {
  return 0;
}

export function c(): number {
  return a();
}
