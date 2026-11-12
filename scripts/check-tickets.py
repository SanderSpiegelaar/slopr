#!/usr/bin/env python3
"""Check docs/tickets against the roadmap, the rule catalog and the gap list.

Usage: check-tickets.py [--clauses | --waves | --place]

--place moves each ticket into docs/tickets/wave-NN/ for its computed wave.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TICKETS = ROOT / "docs/tickets"
REQUIRED = ["id", "title", "kind", "depends_on", "docs", "touches", "verify", "covers"]
SECTIONS = ["## Goal", "## Scope", "## Out of scope", "## Acceptance criteria"]
errors = []


def slug(heading):
    return re.sub(r"[^\w\- ]", "", heading.strip().lower()).replace(" ", "-")


def anchors(path):
    return {slug(m) for m in re.findall(r"^#+\s+(.*)$", path.read_text(), re.M)}


def parse_ticket(path):
    m = re.match(r"---\n(.*?)\n---\n(.*)", path.read_text(), re.S)
    if not m:
        errors.append(f"{path.name}: no frontmatter")
        return None
    meta = {}
    for line in m.group(1).splitlines():
        key, _, value = line.partition(":")
        value = value.strip()
        if value.startswith("[") and value.endswith("]"):
            value = [v.strip() for v in value[1:-1].split(",") if v.strip()]
        meta[key.strip()] = value
    for key in REQUIRED:
        if key not in meta:
            errors.append(f"{path.name}: missing '{key}'")
    positions = [m.group(2).find("\n" + s + "\n") for s in SECTIONS]
    if -1 in positions or positions != sorted(positions):
        errors.append(f"{path.name}: body needs {', '.join(SECTIONS)} in that order")
    if meta.get("id") and path.stem != meta["id"]:
        errors.append(f"{path.name}: file name must be <id>.md")
    meta.setdefault("creates", [])
    return meta


def roadmap_clauses():
    clauses, phase, n = {}, None, 0
    for line in (ROOT / "docs/roadmap.md").read_text().splitlines():
        h = re.match(r"## (p0|v0|v1|v2):", line)
        if h:
            phase, n = h.group(1), 0
        elif line.startswith("## "):
            phase = None
        elif phase and line.startswith("- "):
            n += 1
            clauses[f"{phase}.{n}"] = line[2:]
    return clauses


def catalog_rules():
    rules = {}
    for line in (ROOT / "docs/reference/rules.md").read_text().splitlines():
        m = re.match(r"\| `([a-z]+/[a-z-]+)` \|", line)
        if m:
            phase = re.search(r"\bv([012])\b", line.rstrip(" |").rsplit("|", 1)[-1])
            if phase:
                rules[m.group(1)] = "v" + phase.group(1)
    return rules


def gaps():
    text = (TICKETS / "README.md").read_text()
    return {m.group(1): "resolved" in m.group(2).lower()
            for m in re.finditer(r"^\| (D\d+[ab]?) \|(.*)$", text, re.M)}


def check_links(files):
    for f in files:
        for target in re.findall(r"\]\(([^)\s]+)\)", f.read_text()):
            if target.startswith("http"):
                continue
            path, _, frag = target.partition("#")
            dest = (f.parent / path).resolve() if path else f
            if not dest.exists():
                errors.append(f"{f.relative_to(ROOT)}: broken link {target}")
            elif frag and frag not in anchors(dest):
                errors.append(f"{f.relative_to(ROOT)}: broken anchor {target}")


def load_tickets():
    tickets = {}
    for path in sorted(TICKETS.glob("wave-*/*.md")):
        if t := parse_ticket(path):
            t["path"] = path
            tickets[t["id"]] = t
    return tickets


def check_graph(tickets):
    """Report unknown dependencies and cycles; return each ticket's transitive dependencies."""
    for t in tickets.values():
        for dep in t["depends_on"]:
            if dep not in tickets:
                errors.append(f"{t['id']}: unknown dependency {dep}")

    ancestors, state = {}, {}

    def visit(tid, stack):
        if state.get(tid) == "done":
            return ancestors[tid]
        if state.get(tid) == "open":
            errors.append("cycle: " + " -> ".join(stack + [tid]))
            return set()
        state[tid] = "open"
        found = set()
        for dep in tickets[tid]["depends_on"]:
            if dep in tickets:
                found |= {dep} | visit(dep, stack + [tid])
        state[tid], ancestors[tid] = "done", found
        return found

    for tid in tickets:
        visit(tid, [])
    return ancestors


def check_docs(tickets, ancestors):
    for t in tickets.values():
        inherited = {c for a in ancestors[t["id"]] for c in tickets[a]["creates"]}
        for ref in t["docs"] + t["creates"]:
            path, _, frag = ref.partition("#")
            if not (ROOT / path).exists():
                errors.append(f"{t['id']}: docs file {path} missing")
            elif frag and ref not in t["creates"] and frag not in anchors(ROOT / path) and ref not in inherited:
                errors.append(f"{t['id']}: {ref} neither exists nor is created by a dependency")
        if not t["verify"]:
            errors.append(f"{t['id']}: empty verify")


def check_coverage(tickets, rules, clauses, gap_rows):
    """Each rule, roadmap clause and open gap is covered by exactly one ticket; return the owners."""
    owners = {}
    for t in tickets.values():
        for c in t["covers"]:
            owners.setdefault(c, []).append(t["id"])

    def expect(kind, keys, required):
        for key in keys:
            got = owners.get(f"{kind}:{key}", [])
            if len(got) != (1 if required(key) else 0):
                errors.append(f"{kind}:{key} covered by {got or 'no ticket'}")
        for c in owners:
            if c.startswith(kind + ":") and c.split(":", 1)[1] not in keys:
                errors.append(f"{c} in {owners[c]} does not exist")

    expect("rule", rules, lambda k: True)
    expect("roadmap", clauses, lambda k: True)
    expect("gap", gap_rows, lambda k: not gap_rows[k])
    return owners


def overlap(a, b):
    return a == b or (a.endswith("/") and b.startswith(a)) or (b.endswith("/") and a.startswith(b))


def check_overlaps(tickets, ancestors):
    prs = [t for t in tickets.values() if t["kind"] != "epic"]
    for i, a in enumerate(prs):
        for b in prs[i + 1:]:
            if a["id"] in ancestors[b["id"]] or b["id"] in ancestors[a["id"]]:
                continue
            shared = [x for x in a["touches"] for y in b["touches"] if overlap(x, y)]
            if shared:
                errors.append(f"{a['id']} and {b['id']} both touch {shared[0]} but are not ordered")


def place_waves(tickets, place):
    """Compute each ticket's wave; move misplaced tickets if place, else report them."""
    depth = {}

    def level(tid):
        if tid not in depth:
            depth[tid] = 1 + max((level(d) for d in tickets[tid]["depends_on"]), default=-1)
        return depth[tid]

    for tid in tickets:
        level(tid)
        want = TICKETS / f"wave-{depth[tid]:02d}" / f"{tid}.md"
        if tickets[tid]["path"] != want:
            if place:
                want.parent.mkdir(exist_ok=True)
                tickets[tid]["path"].rename(want)
            else:
                errors.append(f"{tid}: belongs in {want.parent.name}/, run --place")
    if place:
        for d in TICKETS.glob("wave-*"):
            if not any(d.iterdir()):
                d.rmdir()
    return depth


def longest_chain(tickets, depth):
    chain = [max(depth, key=lambda t: (depth[t], t))]
    while tickets[chain[-1]]["depends_on"]:
        chain.append(max(tickets[chain[-1]]["depends_on"], key=lambda d: depth[d]))
    return chain[::-1]


def fail():
    print("\n".join(errors))
    return 1


def main():
    tickets = load_tickets()
    ancestors = check_graph(tickets)
    check_docs(tickets, ancestors)
    rules, clauses, gap_rows = catalog_rules(), roadmap_clauses(), gaps()
    owners = check_coverage(tickets, rules, clauses, gap_rows)
    check_overlaps(tickets, ancestors)
    check_links([ROOT / "AGENTS.md", *ROOT.glob("docs/**/*.md")])
    if errors:
        return fail()

    depth = place_waves(tickets, "--place" in sys.argv)
    if errors:
        return fail()
    chain = longest_chain(tickets, depth)

    if "--clauses" in sys.argv:
        for cid, text in clauses.items():
            print(f"{cid:6} {owners[f'roadmap:{cid}'][0]:12} {text}")
    if "--waves" in sys.argv:
        for w in range(max(depth.values()) + 1):
            print(f"wave {w}: {' '.join(sorted(t for t in depth if depth[t] == w))}")
    print(f"ok: {len(tickets)} tickets, {len(rules)} catalog rules, {len(clauses)} roadmap clauses, "
          f"{len(gap_rows)} gaps; longest chain ({len(chain)}): {' -> '.join(chain)}")
    return 0


sys.exit(main())
