#!/usr/bin/env python3
import argparse
import gzip
import re


CELL_RE = re.compile(r"^(  cell \()([^)]+)(\).*)$")


def strip_cell_underscores(input_path: str, output_path: str) -> int:
    changed = 0

    with gzip.open(input_path, "rt", encoding="utf-8", newline="") as src:
        with gzip.open(output_path, "wt", encoding="utf-8", newline="") as dst:
            for line in src:
                match = CELL_RE.match(line)
                if match:
                    name = match.group(2)
                    stripped = name.replace("_", "")
                    if stripped != name:
                        changed += 1
                        line = f"{match.group(1)}{stripped}{match.group(3)}"
                dst.write(line)

    return changed


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Remove underscores from Liberty cell declaration names."
    )
    parser.add_argument("input", help="Input .lib.gz file")
    parser.add_argument("output", help="Output .lib.gz file")
    args = parser.parse_args()

    changed = strip_cell_underscores(args.input, args.output)
    print(f"rewrote {changed} cell declarations")


if __name__ == "__main__":
    main()
