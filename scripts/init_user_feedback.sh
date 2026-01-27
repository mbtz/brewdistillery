#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: init_user_feedback.sh <file> [--force]

Creates <file> with two lines:
1) Current user feedback
2) A note explaining the user feedback should not be deleted
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

force=false
file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --force)
      force=true
      shift
      ;;
    *)
      if [[ -z $file ]]; then
        file=$1
        shift
      else
        echo "Unexpected argument: $1" >&2
        exit 2
      fi
      ;;
  esac
done

if [[ -z $file ]]; then
  usage
  exit 2
fi

if [[ -e $file && $force == false ]]; then
  echo "File already exists: $file" >&2
  exit 2
fi

python3 - "$file" <<'PY'
import sys
from datetime import datetime

path = sys.argv[1]
user_feedback = sys.stdin.read().strip()
note = "Do not delete the user feedback above; it records the user feedback for this file."

with open(path, "w", encoding="utf-8") as f:
    f.write(user_feedback + "\n")
    f.write(note + "\n")
PY
