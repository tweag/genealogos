use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

use crate::model::Model;
use crate::nixtract::Nixtract;

pub mod model;
pub mod nixtract;

pub fn genealogos(input_entries: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let mut entries = vec![];

    for input_entry in input_entries {
        let entry: nixtract::NixtractEntry =
            serde_json::from_str(input_entry.as_ref().trim()).unwrap();
        entries.push(entry);
    }
    let nixtract: Nixtract = Nixtract { entries };

    let model: Model = nixtract.into();
    let cyclonedx: cyclonedx::CycloneDx = model.into();

    serde_json::to_string(&cyclonedx).unwrap()
}

#[cfg(test)]
mod tests {
    use log::info;
    use std::{fs, io::BufRead};
    use test_log::test;

    #[test]
    fn test_fixtures() {
        let input_dir = fs::read_dir("tests/fixtures/nixtract/success/").unwrap();

        for input_file in input_dir {
            let input_file = input_file.unwrap();
            let input_path = input_file.path();

            if input_path.extension().unwrap().to_string_lossy() == "in" {
                info!("testing: {}", input_path.to_string_lossy());

                let input_file = fs::File::open(&input_path).unwrap();

                let output =
                    crate::genealogos(std::io::BufReader::new(input_file).lines().flatten());

                let mut expected_path = input_path.clone();
                expected_path.set_extension("out");

                let expected_output = fs::read_to_string(expected_path).unwrap();

                assert_eq!(output, expected_output.trim());
            }
        }
    }
}
