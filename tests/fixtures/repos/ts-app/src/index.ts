import { usedHelper } from "./utils";
import { orderTotal } from "./orders";
import { invoiceTotal } from "./invoices";
import { classify } from "./complex";
import { a } from "./cycle-a";
import { c } from "./cycle-b";

import chunk from "lodash/chunk";

console.log(usedHelper(), orderTotal([]), invoiceTotal([]), classify(1, 2, 3), a(), c(), chunk([1, 2], 1));
