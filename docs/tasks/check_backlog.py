"""Backlog consistency check for docs/tasks/*.md (planning tooling, stdlib only).

Usage: python docs/tasks/check_backlog.py [--poirot PATH_TO_POIROT_CHECKOUT]

Fails (exit 1) when a task file is missing a required field, names an unknown dependency or agent,
has a dependency cycle, writes a file that another unordered task also writes, touches a sensitive
path without security-auditor review, writes outside its workstream, reads a repo file produced by a
task it does not depend on, or (with --poirot) cites a Swift path or line range that does not exist.
"""
import glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REQUIRED = ["Goal", "Inputs", "Outputs", "Acceptance criteria", "Dependencies", "Owning agent", "Reviewer agent", "Size"]
AGENTS = {"rust-engineer", "vue-expert", "ui-designer", "frontend-developer", "typescript-pro", "swift-expert", "legacy-modernizer",
          "test-automator", "qa-expert", "security-engineer", "security-auditor", "deployment-engineer", "build-engineer",
          "dependency-manager", "documentation-engineer", "technical-writer", "architect-reviewer", "code-reviewer", "accessibility-tester"}
# Files that exist from planning time; reading them needs no dependency.
PLANNING_DOCS = ("docs/adr/", "docs/ipc-contract.md", "docs/port-plan.md", "docs/parity-matrix.md", "docs/progress-log.md", "docs/tasks/")
# Writing any of these requires security-auditor in the review chain (ADR 0010, port-plan §5.6).
SENSITIVE = ("src-tauri/src/commands/", "src-tauri/capabilities/", "src-tauri/build.rs", "src-tauri/tauri.conf.json", "src/ipc/bindings.ts",
             "crates/heimdall-core/src/config/write/", "crates/heimdall-core/src/data/history_write.rs", "crates/heimdall-core/src/projects/delete.rs",
             "crates/heimdall-core/src/models/secret.rs", "crates/heimdall-core/src/os_integration/", "src/lib/markdown/", ".github/workflows/release.yml")
# Longest prefix wins. Mirrors port-plan §5.3.
OWNERSHIP = {
    "crates/heimdall-core/tests/": "Testing", "crates/heimdall-core/examples/": "Testing", "crates/heimdall-core/Cargo.toml": "Release and packaging",
    "crates/heimdall-core/src/": "Rust data core", "src-tauri/Cargo.toml": "Release and packaging",
    "src-tauri/tauri.windows.conf.json": "Release and packaging", "src-tauri/tauri.linux.conf.json": "Release and packaging",
    "src-tauri/tauri.macos.conf.json": "Release and packaging", "src-tauri/capabilities/": "Security",
    "src-tauri/src/commands/": "Rust data core",
    "src-tauri/src/state.rs": "Rust data core", "src-tauri/src/prefs.rs": "Rust data core", "src-tauri/src/watcher.rs": "Rust data core",
    "src-tauri/src/events.rs": "Rust data core", "src-tauri/tests/": "Rust data core", "src-tauri/": "Tauri shell / OS integration",
    "src/ipc/bindings.ts": "Rust data core", "src/lib/markdown/": "Security", "src/lib/export/": "Feature screens", "src/features/": "Feature screens",
    "src/test/": "Testing", "src/": "Vue shell and design system", "index.html": "Vue shell and design system", "tray.html": "Vue shell and design system",
    "vite.config.ts": "Vue shell and design system", "tsconfig.json": "Vue shell and design system", "tsconfig.node.json": "Vue shell and design system",
    "eslint.config.js": "Vue shell and design system", "vitest.config.ts": "Testing", "playwright.config.ts": "Testing",
    "tests/": "Testing", "e2e/": "Testing", "fixtures/": "Testing", "package.json": "Release and packaging", "package-lock.json": "Release and packaging",
    "Cargo.toml": "Release and packaging", "Cargo.lock": "Release and packaging", "rust-toolchain.toml": "Release and packaging", "deny.toml": "Release and packaging",
    ".github/": "Release and packaging", ".gitignore": "Release and packaging", ".gitattributes": "Release and packaging", ".editorconfig": "Release and packaging",
    "scripts/check-bindings.mjs": "Rust data core", "scripts/lint-security-config": "Security", "scripts/setup-": "Release and packaging", "scripts/": "Testing",
    "tools/golden-oracle/": "Legacy parity extraction", "docs/spec/": "Legacy parity extraction", "docs/design/": "Vue shell and design system",
    "docs/security/command-exposure.md": "Tauri shell / OS integration", "docs/spikes/png-export.md": "Feature screens", "docs/spikes/": "Testing",
    "docs/perf-baseline.md": "Testing", "docs/release/platform-verification.md": "Testing", "docs/release/mvp-signoff.md": "Testing", "docs/release/": "Release and packaging",
    "docs/dev-setup.md": "Release and packaging", "docs/install.md": "Docs", "docs/parity-matrix.md": "Testing", "docs/adr/": "Docs", "docs/port-plan.md": "Docs",
    "AI_INSTRUCTIONS.md": "Docs", "CLAUDE.md": "Docs", "README.md": "Docs", "LICENSE": "Docs", "NOTICE": "Docs",
}


def expand(p):
    m = re.search(r"\{([^}]*)\}", p)
    if not m:
        return [p]
    return [x for opt in m.group(1).split(",") for x in expand(p[:m.start()] + opt + p[m.end():])]


def norm(p):
    p = re.sub(r":[\d,\- ]+$", "", p)
    return p[: p.index("**")] if "**" in p else p


def overlaps(a, b):
    return a == b or (a.endswith("/") and b.startswith(a)) or (b.endswith("/") and a.startswith(b))


def owner_of(path):
    best = max((k for k in OWNERSHIP if path == k or path.startswith(k)), key=len, default=None)
    return OWNERSHIP.get(best)


def parse(path):
    text = open(path, encoding="utf-8").read()
    head = re.match(r"# (M\d-\d\d) · (.+)", text)
    meta = re.search(r"Workstream: (.+?) · Scope: (\w+)", text)
    secs = dict(re.findall(r"^## (.+?)\n(.*?)(?=^## |\Z)", text, re.S | re.M))
    t = {"file": os.path.basename(path), "id": head.group(1) if head else None, "ws": meta.group(1) if meta else None,
         "secs": {k.strip(): v.strip() for k, v in secs.items()}}
    s = t["secs"]
    t["deps"] = re.findall(r"^- (M\d-\d\d) ", s.get("Dependencies", ""), re.M)
    t["inputs"] = [c for c in re.findall(r"^- `([^`]+)`", s.get("Inputs", ""), re.M)]
    t["outputs"] = []
    for code, action in re.findall(r"^- `([^`]+)`(?: — (.*))?$", s.get("Outputs", ""), re.M):
        for p in expand(code):
            t["outputs"].append((norm(p), action or ""))
    t["agents"] = re.findall(r"[a-z]+(?:-[a-z]+)+", s.get("Owning agent", "") + " " + s.get("Reviewer agent", ""))
    return t


def main():
    poirot = sys.argv[sys.argv.index("--poirot") + 1] if "--poirot" in sys.argv else None
    tasks = [parse(f) for f in sorted(glob.glob(os.path.join(HERE, "M*-*.md")))]
    byid = {t["id"]: t for t in tasks}
    errors = []
    for t in tasks:
        for r in REQUIRED:
            if not t["secs"].get(r):
                errors.append(f"{t['file']}: missing or empty '{r}'")
        if t["secs"].get("Size") not in ("S", "M", "L", "XL"):
            errors.append(f"{t['file']}: size must be S, M, L or XL")
        for d in t["deps"]:
            if d not in byid:
                errors.append(f"{t['id']}: unknown dependency {d}")
        for a in t["agents"]:
            if a not in AGENTS:
                errors.append(f"{t['id']}: unknown agent '{a}'")

    anc = {}
    def ancestors(i, stack=()):
        if i in stack:
            errors.append(f"dependency cycle through {i}")
            return set()
        if i not in anc:
            anc[i] = set()
            for d in byid[i]["deps"]:
                if d in byid:
                    anc[i] |= {d} | ancestors(d, stack + (i,))
        return anc[i]
    for i in byid:
        ancestors(i)

    ordered = lambda a, b: a in anc[b] or b in anc[a]
    writers = [(t["id"], p) for t in tasks for p, _ in t["outputs"] if p != "docs/progress-log.md"]
    for x in range(len(writers)):
        for y in range(x + 1, len(writers)):
            (a, pa), (b, pb) = writers[x], writers[y]
            if a != b and overlaps(pa, pb) and not ordered(a, b):
                errors.append(f"parallel write conflict: {a} and {b} both write {pa if len(pa) >= len(pb) else pb}")

    creators = {}
    for t in tasks:
        for p, action in t["outputs"]:
            if action.startswith("create") or action.startswith("fill"):
                creators.setdefault(p, set()).add(t["id"])
    for t in tasks:
        reviewers = t["secs"].get("Reviewer agent", "")
        for p, action in t["outputs"]:
            if p.startswith(SENSITIVE) and "placeholder" not in action and "security-auditor" not in reviewers:
                errors.append(f"{t['id']}: writes sensitive {p} without security-auditor review")
            if "placeholder" not in action and owner_of(p) not in (None, t["ws"]):
                errors.append(f"{t['id']} ({t['ws']}): writes {p}, owned by {owner_of(p)}")
            if owner_of(p) is None:
                errors.append(f"{t['id']}: {p} has no owning workstream in OWNERSHIP")
        for i in t["inputs"]:
            if i.startswith("$POIROT"):
                continue
            p = norm(i.split(" ")[0])
            if p.startswith(PLANNING_DOCS):
                continue
            producers = {c for k, v in creators.items() if overlaps(k, p) for c in v}
            if producers and not producers & (anc[t["id"]] | {t["id"]}):
                errors.append(f"{t['id']}: reads {p} but depends on none of its producers {sorted(producers)}")

    if poirot:
        for t in tasks:
            for i in t["inputs"]:
                if not i.startswith("$POIROT/"):
                    continue
                m = re.match(r"^\$POIROT/([^:\s]*)(?::([\d, \-]+))?", i)
                rel, lines = m.group(1).rstrip(","), (m.group(2) or "")
                found = [f for e in expand(rel) for f in glob.glob(os.path.join(poirot, e))]
                if not found:
                    errors.append(f"{t['id']}: $POIROT path not found: {rel}")
                    continue
                nums = [int(n) for n in re.findall(r"\d+", lines)]
                if nums and os.path.isfile(found[0]):
                    count = sum(1 for _ in open(found[0], encoding="utf-8", errors="replace"))
                    if max(nums) > count:
                        errors.append(f"{t['id']}: {rel} has {count} lines, cited line {max(nums)}")

    for e in errors:
        print("ERROR", e)
    print(f"{len(tasks)} tasks checked, {len(errors)} errors")
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()
