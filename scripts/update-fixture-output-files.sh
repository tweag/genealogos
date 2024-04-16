#!/usr/bin/env bash

if ! builtin type -P "genealogos" &> /dev/null; then
  echo "Genealogos not found in \$PATH, terminating"
  exit 1
fi

for input_file in ./genealogos/tests/fixtures/nixtract/trace-files/*.in; do
  output_file_1_4=${input_file%.in}.1_4.out
  # output_file_1_5=${input_file%.in}.1_5.out
  echo "Updating: $output_file_1_4"
  GENEALOGOS_DETERMINISTIC=1 genealogos --file "$input_file" -o "$output_file_1_4" --bom cyclonedx_1.4_json
  # TODO: Update to 1.5
  # echo "Updating: $output_file_1_5"
  # GENEALOGOS_DETERMINISTIC=1 genealogos --file "$input_file" "$output_file_1_5" --cyclonedx-version 1.5
done

for input_file in ./genealogos/tests/fixtures/nixtract/flakes/*.in; do
  output_file_1_4=${input_file%.in}.1_4.out
  # output_file_1_5=${input_file%.in}.1_5.out
  flake_ref=$(jq -r .flake_ref < "$input_file")
  attribute_path=$(jq -r .attribute_path < "$input_file")
  echo "Updating: $output_file_1_4"
  GENEALOGOS_DETERMINISTIC=1 genealogos --flake-ref "$flake_ref" --attribute-path "$attribute_path" -o "$output_file_1_4" --bom cyclonedx_1.4_json
  # echo "Updating: $output_file_1_5"
  # GENEALOGOS_DETERMINISTIC=1 genealogos --flake-ref "$flake_ref" --attribute-path "$attribute_path" "$output_file_1_5" --cyclonedx-version 1.5
done
