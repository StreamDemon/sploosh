#!/usr/bin/env python3
"""Enforce per-file token budgets on the PROMPT-edition spec mirrors.

Runs in CI per §1 principle 7 of LANGUAGE_SPEC.md. Encodes the two prompt
artifacts with cl100k_base and applies a three-tier rule:

  * < warn-at fraction of ceiling -> pass silently
  * >= warn-at fraction and <= 100%  -> warn (exit 0)
  * > 100% of ceiling                -> fail (exit 1)

Defaults match the v0.5.9 budgets: _CORE = 4800, _WEB3 = 1500, warn at 90%.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import tiktoken

REPO_ROOT = Path(__file__).resolve().parent.parent
CORE_PATH = REPO_ROOT / "docs" / "spec-plans" / "LANGUAGE_SPEC_PROMPT_CORE.md"
WEB3_PATH = REPO_ROOT / "docs" / "spec-plans" / "LANGUAGE_SPEC_PROMPT_WEB3.md"


def count_tokens(path: Path, enc: "tiktoken.Encoding") -> int:
    return len(enc.encode(path.read_text(encoding="utf-8")))


def classify(tokens: int, ceiling: int, warn_at: float) -> str:
    if tokens > ceiling:
        return "fail"
    if tokens >= ceiling * warn_at:
        return "warn"
    return "ok"


def report(label: str, tokens: int, ceiling: int, status: str) -> str:
    pct = (tokens / ceiling) * 100.0
    prefix = {"ok": "OK", "warn": "WARN", "fail": "FAIL"}[status]
    return f"{prefix}: {label} at {pct:.1f}% of budget ({tokens}/{ceiling})"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--core-ceiling", type=int, default=4800)
    parser.add_argument("--web3-ceiling", type=int, default=1500)
    parser.add_argument("--warn-at", type=float, default=0.9,
                        help="Warn threshold as a fraction of the ceiling (default: 0.9)")
    args = parser.parse_args(argv)

    enc = tiktoken.get_encoding("cl100k_base")

    targets = [
        ("_CORE", CORE_PATH, args.core_ceiling),
        ("_WEB3", WEB3_PATH, args.web3_ceiling),
    ]

    failed = False
    for label, path, ceiling in targets:
        if not path.is_file():
            print(f"FAIL: {label} missing at {path}")
            failed = True
            continue
        tokens = count_tokens(path, enc)
        status = classify(tokens, ceiling, args.warn_at)
        if status != "ok":
            print(report(label, tokens, ceiling, status))
        if status == "fail":
            failed = True

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
