const admin = "user​"; // expect: text/invisible-unicode
const label = "safe‮⁦txt⁩‬"; // expect: text/invisible-unicode
const joined = "a‍b⁠c"; // expect: text/invisible-unicode
const marked = "﻿inner"; // expect: text/invisible-unicode
const plain = "visible";

export { admin, label, joined, marked, plain };
