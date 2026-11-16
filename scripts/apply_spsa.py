#!/usr/bin/env python3
"""Apply OpenBench SPSA output to the tunable defaults in tuneable.rs.

Input is one `name, value` pair per line (the final values from an SPSA run):

    lmp_base, 4
    see_pawn, 97

Only the default value of each matching `tunable_params!` entry is rewritten;
ranges, the SPSA flag, and column alignment are preserved.

Usage:
    scripts/apply_spsa.py spsa_output.txt
    scripts/apply_spsa.py < spsa_output.txt
    scripts/apply_spsa.py --dry-run spsa_output.txt
"""

import argparse
import re
import sys
from pathlib import Path

DEFAULT_TARGET = Path(__file__).resolve().parent.parent / "crates/engine/src/tuneable.rs"

# name = value, min..=max,<padding>flag;
ENTRY_RE = re.compile(
    r"^(?P<indent>\s*)(?P<name>[a-z_0-9]+)(?P<eq>\s*=\s*)(?P<val>-?\d+)"
    r"(?P<sep>,\s*)(?P<min>-?\d+)\.\.=(?P<max>-?\d+),(?P<pad>\s*)(?P<flag>true|false);\s*$"
)


def parse_spsa(text: str) -> dict[str, int]:
    values: dict[str, int] = {}
    for lineno, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split(",")]
        if len(parts) < 2:
            raise ValueError(f"line {lineno}: expected 'name, value', got {raw!r}")
        name, value = parts[0], parts[1]
        try:
            # OpenBench may report values as floats (e.g. "97.0" or "96.6").
            number = round(float(value))
        except ValueError:
            raise ValueError(f"line {lineno}: bad value {value!r} for {name}") from None
        if name in values:
            raise ValueError(f"line {lineno}: duplicate parameter {name}")
        values[name] = number
    return values


def apply(source: str, values: dict[str, int]) -> tuple[str, list[str], list[str], list[str]]:
    """Return (new_source, changed, out_of_range, missing_from_input)."""
    remaining = dict(values)
    changed: list[str] = []
    out_of_range: list[str] = []
    missing: list[str] = []
    out_lines: list[str] = []

    for line in source.splitlines(keepends=True):
        eol = line[len(line.rstrip("\r\n")):]
        m = ENTRY_RE.match(line.rstrip("\r\n"))
        if not m:
            out_lines.append(line)
            continue

        name = m["name"]
        if name not in remaining:
            if m["flag"] == "true":
                missing.append(name)
            out_lines.append(line)
            continue

        new = remaining.pop(name)
        lo, hi = int(m["min"]), int(m["max"])
        if not lo <= new <= hi:
            out_of_range.append(f"{name} = {new} (range {lo}..={hi})")
        if new != int(m["val"]):
            changed.append(f"{name}: {m['val']} -> {new}")

        # Re-pad so the trailing flag stays in its original column.
        flag_col = len(line.rstrip("\r\n")) - len(f"{m['flag']};")
        head = f"{m['indent']}{name}{m['eq']}{new}{m['sep']}{lo}..={hi},"
        pad = " " * max(1, flag_col - len(head))
        out_lines.append(f"{head}{pad}{m['flag']};{eol}")

    if remaining:
        raise KeyError(", ".join(sorted(remaining)))
    return "".join(out_lines), changed, out_of_range, missing


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("input", nargs="?", help="SPSA output file (default: stdin)")
    ap.add_argument("--target", type=Path, default=DEFAULT_TARGET, help="tuneable.rs to update")
    ap.add_argument("--dry-run", action="store_true", help="report changes without writing")
    args = ap.parse_args()

    text = Path(args.input).read_text() if args.input else sys.stdin.read()
    try:
        values = parse_spsa(text)
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    try:
        new_source, changed, out_of_range, missing = apply(args.target.read_text(), values)
    except KeyError as e:
        print(f"error: parameters not found in {args.target}: {e.args[0]}", file=sys.stderr)
        return 1

    if out_of_range:
        print("error: values outside their declared range (the build would fail):", file=sys.stderr)
        for item in out_of_range:
            print(f"  {item}", file=sys.stderr)
        return 1

    for item in changed:
        print(item)
    print(f"{len(changed)} changed, {len(values) - len(changed)} unchanged")
    if missing:
        print(f"warning: SPSA-flagged params absent from input: {', '.join(missing)}", file=sys.stderr)

    if not args.dry_run:
        args.target.write_text(new_source)
    return 0


if __name__ == "__main__":
    sys.exit(main())
