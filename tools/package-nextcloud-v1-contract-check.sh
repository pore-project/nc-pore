#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="$(mktemp -d)"
trap 'rm -rf "${OUTPUT_DIR}"' EXIT
RUNTIME_BINARY="${ROOT_DIR}/target/release/pore-runtime"
ARCHIVE="${OUTPUT_DIR}/pore.tar.gz"

bash "${ROOT_DIR}/tools/package-nextcloud-v1.sh" "${OUTPUT_DIR}" "${RUNTIME_BINARY}"

if [[ ! -f "${ARCHIVE}" ]]; then
  echo "V1 package was not created: ${ARCHIVE}" >&2
  exit 1
fi

mapfile -t top_levels < <(tar -tzf "${ARCHIVE}" | cut -d/ -f1 | sort -u)
if (( ${#top_levels[@]} != 1 )) || [[ "${top_levels[0]}" != "pore" ]]; then
  echo "V1 package must contain exactly one top-level directory named pore." >&2
  printf "Observed top-level entries:\n%s\n" "${top_levels[*]-<none>}" >&2
  exit 1
fi

required_entries=(
  "pore/appinfo/info.xml"
  "pore/js/pore-browser-completion-job.js"
  "pore/lib/AppInfo/Application.php"
  "pore/runtime/bin/pore-runtime"
)

for entry in "${required_entries[@]}"; do
  if ! tar -tzf "${ARCHIVE}" | grep -Fxq "${entry}"; then
    echo "Required V1 package entry is missing: ${entry}" >&2
    exit 1
  fi
done

if tar -tzf "${ARCHIVE}" | grep -E "\.test\.js$" >/dev/null; then
  echo "V1 production package must not contain JavaScript test files." >&2
  tar -tzf "${ARCHIVE}" | grep -E "\.test\.js$" >&2
  exit 1
fi

if ! git -C "${ROOT_DIR}" ls-files "web/*.test.js" | grep -q .; then
  echo "Repository test files disappeared from source control." >&2
  exit 1
fi

if ! grep -Fq "../web/pore-browser-completion-job.test.js" "${ROOT_DIR}/tools/js-test-harness.mjs"; then
  echo "Representative browser test is no longer registered in the JavaScript test harness." >&2
  exit 1
fi

echo "V1 package contract passed."
