#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${1:-${ROOT_DIR}/build/artifacts}"
RUNTIME_BINARY="${2:-${ROOT_DIR}/target/release/pore-runtime}"
APP_NAME="pore"

if [[ ! -x "${RUNTIME_BINARY}" ]]; then
  echo "PoRE runtime binary is missing or not executable: ${RUNTIME_BINARY}" >&2
  echo "Build it first with: cargo build --release -p pore-runtime" >&2
  exit 1
fi

rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}"
STAGE_DIR="$(mktemp -d)"
trap 'rm -rf "${STAGE_DIR}"' EXIT

APP_DIR="${STAGE_DIR}/${APP_NAME}"
mkdir -p "${APP_DIR}"

PACKAGE_PATHS="${STAGE_DIR}/package-paths"
git -C "${ROOT_DIR}" ls-files -z appinfo css js lib web |
  grep -zEv '\.test\.js$' > "${PACKAGE_PATHS}"

git -C "${ROOT_DIR}" archive --format=tar --pathspec-from-file="${PACKAGE_PATHS}" --pathspec-file-nul HEAD |
  tar -x -C "${APP_DIR}"

mkdir -p "${APP_DIR}/runtime/bin"
install -m 0755 "${RUNTIME_BINARY}" "${APP_DIR}/runtime/bin/pore-runtime"

tar -C "${STAGE_DIR}" -czf "${OUTPUT_DIR}/${APP_NAME}.tar.gz" "${APP_NAME}"

echo "Created ${OUTPUT_DIR}/${APP_NAME}.tar.gz"
