#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: update_user_feedback.sh <file>

Replaces the first line of <file> with the current user feedback.
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 1 ]]; then
  usage
  exit 2
fi

file=$1

if [[ ! -f $file ]]; then
  echo "File not found: $file" >&2
  exit 2
fi

python3 - "$file" <<'PY'
import sys
from datetime import datetime

path = sys.argv[1]
timestamp = datetime.now().strftime("%Y-%m-%dT%H:%M:%S")

with open(path, "r", encoding="utf-8", errors="replace") as f:
    lines = f.readlines()

if not lines:
    lines = [timestamp + "\n"]
else:
    newline = "\n" if lines[0].endswith("\n") else ""
    lines[0] = timestamp + newline

with open(path, "w", encoding="utf-8") as f:
    f.writelines(lines)
PY
