#!/usr/bin/env bash

if ! builtin type -P "genealogos" &> /dev/null; then
  echo "Genealogos not found in \$PATH, terminating"
  exit 1
fi

for input_file in ./genealogos/tests/fixtures/nixtract/success/*.in; do
  output_file=${input_file%.in}.out
  GENEALOGOS_DETERMINISTIC=1 genealogos "$input_file" > "$output_file"
done
