#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: check_user_feedback.sh <file> [--threshold <seconds>]

Checks whether the user feedback in <file> is up to date with the current user feedback.

Outputs "true" if up to date, "false" otherwise.
Exit code: 0 when "true", 1 when "false".
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -lt 1 ]]; then
  usage
  exit 2
fi

file=$1
shift

threshold=1
if [[ ${1:-} == "--threshold" ]]; then
  if [[ -z ${2:-} ]]; then
    echo "Missing value for --threshold" >&2
    exit 2
  fi
  threshold=$2
  shift 2
fi

if [[ $# -gt 0 ]]; then
  echo "Unexpected argument(s): $*" >&2
  exit 2
fi

if [[ ! -f $file ]]; then
  echo "File not found: $file" >&2
  exit 2
fi

python3 - "$file" "$threshold" <<'PY'
import os
import re
import sys
import time
from datetime import datetime

path = sys.argv[1]
threshold = float(sys.argv[2])

with open(path, "r", encoding="utf-8", errors="replace") as f:
    first_line = f.readline().strip()

def parse_timestamp(line: str) -> float:
    line = line.strip()
    if not line:
        raise ValueError("First line is empty")

    # Pure epoch seconds (int or float)
    if re.fullmatch(r"\d+(?:\.\d+)?", line):
        return float(line)

    # ISO-8601 timestamp anywhere in the line
    m = re.search(
        r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)",
        line,
    )
    if m:
        token = m.group(1).replace(" ", "T")
        if token.endswith("Z"):
            token = token[:-1] + "+00:00"
        # Normalize timezone like +0000 to +00:00 for fromisoformat
        if re.search(r"[+-]\d{4}$", token):
            token = token[:-5] + token[-5:-2] + ":" + token[-2:]
        dt = datetime.fromisoformat(token)
        if dt.tzinfo is None:
            return time.mktime(dt.timetuple()) + dt.microsecond / 1_000_000
        return dt.timestamp()

    # Fallback to date-only ISO string
    m = re.search(r"(\d{4}-\d{2}-\d{2})", line)
    if m:
        dt = datetime.fromisoformat(m.group(1))
        return time.mktime(dt.timetuple())

    raise ValueError(f"Unrecognized timestamp: {line!r}")

try:
    ts = parse_timestamp(first_line)
except Exception as exc:
    print(f"false", end="")
    sys.stderr.write(f"Error: {exc}\n")
    sys.exit(2)

mtime = os.path.getmtime(path)
is_newer = mtime > (ts + threshold)

print("true" if is_newer else "false", end="")
sys.exit(0 if is_newer else 1)
PY
