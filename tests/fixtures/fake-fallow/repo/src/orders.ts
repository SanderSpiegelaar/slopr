interface Item {
  price: number;
  qty: number;
  taxable: boolean;
}

export function orderTotal(items: Item[]): number {
  let total = 0;
  for (const item of items) {
    const line = item.price * item.qty;
    total += line;
  }
  let tax = 0;
  for (const item of items) {
    if (item.taxable) {
      tax += item.price * item.qty * 0.21;
    }
  }
  const shipping = total > 100 ? 0 : 5;
  return total + tax + shipping;
}
