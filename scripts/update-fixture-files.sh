#!/usr/bin/env bash

if ! builtin type -P "genealogos" &> /dev/null; then
  echo "Genealogos not found in \$PATH, terminating"
  exit 1
fi

for input_file in ./genealogos/tests/fixtures/nixtract/trace-files/*.in; do
  output_file=${input_file%.in}.out
  echo "Updating: $output_file"
  GENEALOGOS_DETERMINISTIC=1 genealogos --file "$input_file" "$output_file"
done

for input_file in ./genealogos/tests/fixtures/nixtract/flakes/*.in; do
  output_file=${input_file%.in}.out
  flake_ref=$(jq -r .flake_ref < "$input_file")
  attribute_path=$(jq -r .attribute_path < "$input_file")
  echo "Updating: $output_file"
  GENEALOGOS_DETERMINISTIC=1 genealogos --flake-ref "$flake_ref" --attribute-path "$attribute_path" "$output_file"
done
