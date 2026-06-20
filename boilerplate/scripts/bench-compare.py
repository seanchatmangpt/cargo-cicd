#!/usr/bin/env python3
"""bench-compare.py — Compare two criterion bench output files and report regressions.

Usage:
    python3 bench-compare.py baseline.txt current.txt

Output:
    A Markdown table printed to stdout:

    | Benchmark | Baseline | Current | Change |
    |-----------|----------|---------|--------|
    | foo       | 1,234 ns | 1,300 ns | +5.35% ⚠️ |
    | bar       | 500 ns   | 480 ns  | -4.00% ✅ |

Exit codes:
    0 — all benchmarks within the 10% regression threshold (or no regressions found)
    1 — at least one benchmark regressed by more than 10%

Supported input formats:
    Criterion libtest format:
        test bench_name ... bench:       1,234 ns/iter (+/- 56)

    Criterion HTML/progress summary lines (best-estimate row):
        bench_name   time:   [1.2345 ms 1.2350 ms 1.2355 ms]

Both formats may appear in the same file; unrecognised lines are ignored.
"""

import re
import sys
from pathlib import Path
from typing import Optional

# ── Regexes for the two common criterion output formats ───────────────────────

# libtest-style:  test some_bench ... bench:   1,234 ns/iter (+/- 56)
_RE_LIBTEST = re.compile(
    r"^test\s+(\S+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+ns/iter",
    re.IGNORECASE,
)

# Criterion summary-style:  some_bench   time:   [... 1.2350 ms ...]
# The middle value of the three-element confidence interval is the point estimate.
_RE_CRITERION = re.compile(
    r"^(\S+)\s+time:\s+\[[\d.]+\s+\w+\s+([\d.]+)\s+(\w+)\s+[\d.]+\s+\w+\]",
    re.IGNORECASE,
)

# Unit multipliers → nanoseconds
_UNIT_NS: dict[str, float] = {
    "ns": 1.0,
    "µs": 1_000.0,
    "us": 1_000.0,
    "ms": 1_000_000.0,
    "s":  1_000_000_000.0,
}

REGRESSION_THRESHOLD_PCT: float = 10.0


def _strip_commas(s: str) -> str:
    return s.replace(",", "")


def parse_bench_output(path: Path) -> dict[str, float]:
    """Return {bench_name: time_ns} for every benchmark found in *path*.

    If the file does not exist or is empty, returns an empty dict without
    raising — the caller handles the "no baseline" case gracefully.
    """
    results: dict[str, float] = {}
    if not path.exists():
        return results

    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()

        m = _RE_LIBTEST.match(line)
        if m:
            name = m.group(1)
            ns = float(_strip_commas(m.group(2)))
            results[name] = ns
            continue

        m = _RE_CRITERION.match(line)
        if m:
            name = m.group(1)
            value = float(m.group(2))
            unit = m.group(3).lower()
            multiplier = _UNIT_NS.get(unit, 1.0)
            results[name] = value * multiplier

    return results


def _format_ns(ns: float) -> str:
    """Human-readable time string from a nanosecond value."""
    if ns >= 1_000_000_000:
        return f"{ns / 1_000_000_000:.3f} s"
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.3f} µs"
    return f"{ns:,.0f} ns"


def build_markdown_table(
    baseline: dict[str, float],
    current: dict[str, float],
) -> tuple[str, bool]:
    """Return (markdown_table, has_regression).

    *has_regression* is True if any benchmark regressed by more than
    REGRESSION_THRESHOLD_PCT percent.
    """
    # All names from both runs (deterministic order)
    all_names = sorted(set(baseline) | set(current))

    if not all_names:
        return "_No benchmark results found in either file._", False

    rows: list[str] = []
    has_regression = False

    header = "| Benchmark | Baseline | Current | Change |"
    separator = "|-----------|----------|---------|--------|"

    for name in all_names:
        b = baseline.get(name)
        c = current.get(name)

        if b is None:
            # New benchmark — no baseline to compare against
            rows.append(
                f"| `{name}` | _new_ | {_format_ns(c)} | — (new) |"  # type: ignore[arg-type]
            )
            continue

        if c is None:
            # Benchmark removed
            rows.append(
                f"| `{name}` | {_format_ns(b)} | _removed_ | — (removed) |"
            )
            continue

        pct = (c - b) / b * 100 if b != 0 else 0.0
        sign = "+" if pct >= 0 else ""
        change_str = f"{sign}{pct:.2f}%"

        if pct > REGRESSION_THRESHOLD_PCT:
            has_regression = True
            indicator = " ⚠️ REGRESSION"
        elif pct < -REGRESSION_THRESHOLD_PCT:
            indicator = " ✅ improvement"
        elif pct > 0:
            indicator = " ⚠️ slower"
        else:
            indicator = " ✅"

        rows.append(
            f"| `{name}` | {_format_ns(b)} | {_format_ns(c)} | {change_str}{indicator} |"
        )

    table_lines = [header, separator] + rows
    return "\n".join(table_lines), has_regression


def _regression_summary(has_regression: bool) -> str:
    if has_regression:
        return (
            f"\n> **⚠️ One or more benchmarks regressed by more than "
            f"{REGRESSION_THRESHOLD_PCT:.0f}%.** Review the table above."
        )
    return f"\n> All benchmarks are within the {REGRESSION_THRESHOLD_PCT:.0f}% regression threshold. ✅"


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(
            f"Usage: {argv[0]} baseline.txt current.txt",
            file=sys.stderr,
        )
        return 2

    baseline_path = Path(argv[1])
    current_path = Path(argv[2])

    if not current_path.exists():
        print(f"Error: current benchmark file not found: {current_path}", file=sys.stderr)
        return 2

    baseline = parse_bench_output(baseline_path)
    current = parse_bench_output(current_path)

    if not baseline:
        print(
            "_No prior baseline available — this run will become the new baseline._\n",
        )
        # Still emit a table of current results for visibility.
        baseline = {}  # empty; all results appear as "new"

    table, has_regression = build_markdown_table(baseline, current)

    print(table)
    print(_regression_summary(has_regression))

    return 1 if has_regression else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
