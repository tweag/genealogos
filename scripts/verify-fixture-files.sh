#!/usr/bin/env bash

if ! builtin type -P "cyclonedx" &> /dev/null; then
  echo "cyclonedx not found in \$PATH, terminating"
  exit 1
fi

for f in ./genealogos/tests/fixtures/nixtract/trace-files/*.out; do
  echo "$f"
  OUT=$(cyclonedx validate --input-format json --input-version v1_5 --input-file "$f")
  echo "$OUT"
  # Fail if the cyclonedx tool did not output a message containing "successfully"
  [[ "$OUT" =~ .*successfully.* ]] || exit 1
done
