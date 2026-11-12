export function classify(a: number, b: number, c: number): string {
  let out = "";
  for (let i = 0; i < a; i++) {
    if (i % 2 === 0) {
      if (b > 0) {
        for (let j = 0; j < b; j++) {
          if (j % 3 === 0 && c > 1) {
            out += "a";
          } else if (j % 5 === 0 || c < -1) {
            out += "b";
          } else {
            while (c > 10) {
              if (c % 2 === 0) {
                c -= 3;
              } else if (c % 7 === 0) {
                c -= 1;
              } else {
                c -= 2;
              }
            }
          }
        }
      } else if (b < -5) {
        out += "c";
      } else {
        out += a > b ? "d" : "e";
      }
    } else {
      switch (c) {
        case 1:
          out += "f";
          break;
        case 2:
          out += "g";
          break;
        default:
          out += c > 0 && b > 0 ? "h" : "i";
      }
    }
  }
  return out;
}
