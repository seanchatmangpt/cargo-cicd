#!/usr/bin/env python3
"""
mutation-report.py — Parse cargo-mutants output and report the mutation score.

Usage:
    python3 scripts/mutation-report.py [--threshold 70] [--input path/to/summary.json]

Exit codes:
    0  Mutation score >= threshold (default 70 %)
    1  Mutation score < threshold
    2  summary.json not found or could not be parsed
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


# ---------------------------------------------------------------------------
# Colour helpers — disabled automatically when stdout is not a TTY
# ---------------------------------------------------------------------------

_USE_COLOR = sys.stdout.isatty()


def _c(code: str, text: str) -> str:
    if not _USE_COLOR:
        return text
    return f"\033[{code}m{text}\033[0m"


def red(t: str) -> str:
    return _c("31", t)


def green(t: str) -> str:
    return _c("32", t)


def yellow(t: str) -> str:
    return _c("33", t)


def bold(t: str) -> str:
    return _c("1", t)


def dim(t: str) -> str:
    return _c("2", t)


# ---------------------------------------------------------------------------
# Formatting helpers
# ---------------------------------------------------------------------------

_BAR_WIDTH = 30


def _score_bar(score: float) -> str:
    """Return a simple ASCII progress bar representing the score (0–100)."""
    filled = int(round(score / 100 * _BAR_WIDTH))
    bar = "#" * filled + "-" * (_BAR_WIDTH - filled)
    if score >= 80:
        bar = green(bar)
    elif score >= 60:
        bar = yellow(bar)
    else:
        bar = red(bar)
    return f"[{bar}]"


def _score_label(score: float, threshold: float) -> str:
    label = f"{score:.1f}%"
    if score >= threshold:
        return green(bold(label))
    return red(bold(label))


# ---------------------------------------------------------------------------
# Report logic
# ---------------------------------------------------------------------------

def _load_summary(path: Path) -> dict:
    try:
        with path.open() as fh:
            return json.load(fh)
    except FileNotFoundError:
        print(red(f"Error: summary.json not found at {path}"), file=sys.stderr)
        print(
            dim("  Run `cargo mutants --workspace --output target/mutants` first."),
            file=sys.stderr,
        )
        sys.exit(2)
    except json.JSONDecodeError as exc:
        print(red(f"Error: could not parse {path}: {exc}"), file=sys.stderr)
        sys.exit(2)


def _top_missed(data: dict, n: int = 5) -> list[str]:
    """Return up to n descriptions of missed mutants (best-effort)."""
    missed_list: list[dict] = data.get("missed_mutants", [])
    # cargo-mutants summary.json schema varies; try common field names.
    results: list[str] = []
    for m in missed_list[:n]:
        desc = (
            m.get("description")
            or m.get("name")
            or m.get("mutation")
            or str(m)
        )
        file_loc = m.get("file") or m.get("source_file") or ""
        line = m.get("line") or ""
        location = f" ({file_loc}:{line})" if file_loc else ""
        results.append(f"{desc}{location}")
    return results


def _print_report(data: dict, threshold: float) -> float:
    caught: int = data.get("caught", 0)
    missed: int = data.get("missed", 0)
    timeout: int = data.get("timeout", 0)
    unviable: int = data.get("unviable", 0)
    total: int = caught + missed + timeout

    score: float = (caught / total * 100) if total > 0 else 0.0

    # ------------------------------------------------------------------
    # Header
    # ------------------------------------------------------------------
    print()
    print(bold("Mutation Testing Report"))
    print("=" * 42)

    # ------------------------------------------------------------------
    # Score bar
    # ------------------------------------------------------------------
    print(f"  Score   {_score_bar(score)}  {_score_label(score, threshold)}")
    print(f"  Target  {dim(f'{threshold:.0f}%')}")
    print()

    # ------------------------------------------------------------------
    # Counts table
    # ------------------------------------------------------------------
    rows = [
        ("Caught",   caught,   green),
        ("Missed",   missed,   red if missed > 0 else dim),
        ("Timeout",  timeout,  yellow if timeout > 0 else dim),
        ("Unviable", unviable, dim),
        ("Total",    total,    bold),
    ]
    col_w = max(len(label) for label, *_ in rows)
    for label, count, colourise in rows:
        count_str = colourise(str(count))
        print(f"  {label:<{col_w}}  {count_str}")
    print()

    # ------------------------------------------------------------------
    # Top missed mutants
    # ------------------------------------------------------------------
    missed_descriptions = _top_missed(data)
    if missed_descriptions:
        print(bold("Top missed mutations (up to 5):"))
        for i, desc in enumerate(missed_descriptions, start=1):
            print(f"  {dim(str(i) + '.')} {desc}")
        print()
    elif missed > 0:
        # summary.json doesn't include per-mutant detail — guide the user.
        print(
            dim(
                "  Per-mutant detail not available in summary.json.\n"
                "  Run with `--output target/mutants` and inspect missed/ directory."
            )
        )
        print()

    # ------------------------------------------------------------------
    # Verdict line
    # ------------------------------------------------------------------
    if total == 0:
        print(yellow("No mutants were generated — nothing to report."))
    elif score >= threshold:
        print(green(f"PASS  Mutation score {score:.1f}% meets the {threshold:.0f}% threshold."))
    else:
        print(
            red(
                f"FAIL  Mutation score {score:.1f}% is below the {threshold:.0f}% threshold."
            )
        )
    print()

    return score


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Parse cargo-mutants summary.json and report the mutation score.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=70.0,
        metavar="PCT",
        help="Minimum acceptable mutation score in percent (default: 70).",
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=Path("target/mutants/summary.json"),
        metavar="PATH",
        help="Path to summary.json (default: target/mutants/summary.json).",
    )
    args = parser.parse_args()

    data = _load_summary(args.input)
    score = _print_report(data, args.threshold)

    caught = data.get("caught", 0)
    missed = data.get("missed", 0)
    timeout = data.get("timeout", 0)
    total = caught + missed + timeout

    if total == 0:
        # No mutants generated — treat as inconclusive, not a failure.
        sys.exit(0)

    sys.exit(0 if score >= args.threshold else 1)


if __name__ == "__main__":
    main()
