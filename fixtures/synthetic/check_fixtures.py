#!/usr/bin/env python3
"""Acceptance checks for fixtures/synthetic (task M0-11).

Run from anywhere; paths are resolved relative to this file.

Checks:
  1. Every .json file outside corrupt/ (and not on the documented invalid list) parses as JSON.
  2. Every .jsonl line outside corrupt/ (and not on the documented invalid list) parses as JSON.
  3. No absolute home path other than /Users/fixture appears anywhere under this tree.

Exits 0 on success, 1 on the first category of failure (printing every offender first).
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
CORRUPT_DIR = os.path.join(ROOT, "corrupt")

# Files that are valid-JSON-parseable containers but contain a deliberately
# invalid line/value, documented in README.md's "Intentionally invalid
# fixtures" section. These are skipped by the strict per-file JSON check but
# still checked line-by-line where useful (see INVALID_LINE_OK below).
INVALID_JSON_FILES = {
    os.path.join(ROOT, "cases", "FacetsLoaderTests", "loadFacets_invalidJSON_returnsNil", "bad-session.json"),
    os.path.join(ROOT, "cases", "FacetsLoaderTests", "loadAllFacets_skipsInvalidFiles", "bad-session.json"),
    os.path.join(ROOT, "cases", "TodoLoaderTests", "loadTodos_skipsInvalidJSON", "abc-1234-agent-bad.json"),
}
INVALID_JSONL_FILES = {
    os.path.join(ROOT, "cases", "HistoryLoaderTests", "loadAll_skipsInvalidLines", "history.jsonl"),
    os.path.join(ROOT, "cases", "TranscriptParserTests", "parse_malformedLines_skipsGracefully",
                 "test-session.jsonl"),
}

errors = []

# --- 1 & 2: JSON / JSONL validity ---------------------------------------
for dirpath, dirnames, filenames in os.walk(ROOT):
    if os.path.commonpath([dirpath, CORRUPT_DIR]) == CORRUPT_DIR:
        continue  # inside corrupt/
    for name in filenames:
        full = os.path.join(dirpath, name)
        if name.endswith(".json"):
            if full in INVALID_JSON_FILES:
                continue
            with open(full, "r", encoding="utf-8") as f:
                text = f.read()
            if text.strip() == "":
                continue  # e.g. loadAll_emptyFile_returnsEmpty's history.jsonl sibling cases
            try:
                json.loads(text)
            except json.JSONDecodeError as e:
                errors.append(f"invalid JSON: {full}: {e}")
        elif name.endswith(".jsonl"):
            if full in INVALID_JSONL_FILES:
                continue
            with open(full, "r", encoding="utf-8") as f:
                for i, line in enumerate(f, start=1):
                    line = line.strip("\n").strip("\r")
                    if not line:
                        continue
                    try:
                        json.loads(line)
                    except json.JSONDecodeError as e:
                        errors.append(f"invalid JSONL line: {full}:{i}: {e}")

if errors:
    print(f"JSON/JSONL validity: {len(errors)} problem(s)")
    for e in errors:
        print(" -", e)
else:
    print("JSON/JSONL validity: OK")

# --- 3: home path check ---------------------------------------------------
BAD_HOME_PATTERNS = [
    re.compile(r"/Users/(?!fixture\b)[A-Za-z0-9_.\-]+"),
    re.compile(r"C:\\\\Users\\\\(?!fixture\b)[A-Za-z0-9_.\-]+"),
    re.compile(r"/home/(?!fixture\b)[A-Za-z0-9_.\-]+"),
]
home_errors = []
for dirpath, dirnames, filenames in os.walk(ROOT):
    for name in filenames:
        if name == "check_fixtures.py":
            continue
        full = os.path.join(dirpath, name)
        try:
            with open(full, "r", encoding="utf-8") as f:
                text = f.read()
        except (UnicodeDecodeError, PermissionError):
            continue
        for pat in BAD_HOME_PATTERNS:
            for m in pat.finditer(text):
                home_errors.append(f"{full}: {m.group(0)}")

if home_errors:
    print(f"home path check: {len(home_errors)} problem(s)")
    for e in sorted(set(home_errors)):
        print(" -", e)
else:
    print("home path check: OK (/Users/fixture only)")

if errors or home_errors:
    sys.exit(1)
print("all checks passed")
