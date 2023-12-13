mod model;
mod nixtract;

use std::io::{self, BufRead};

use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

use crate::model::Model;
use crate::nixtract::Nixtract;

fn main() -> Result<(), io::Error> {
    let mut input_reader = std::io::stdin().lock();
    let mut entries = vec![];

    loop {
        let mut buffer = String::new();
        match input_reader.read_line(&mut buffer) {
            Ok(0) => break,
            Ok(_n) => {
                let entry: nixtract::NixtractEntry = serde_json::from_str(&buffer.trim()).unwrap();
                entries.push(entry);
            }
            Err(_) => todo!(),
        }
    }
    let nixtract: Nixtract = Nixtract { entries };

    let model: Model = nixtract.into();
    let cyclonedx: cyclonedx::CycloneDx = model.into();
    let json_out = serde_json::to_string(&cyclonedx).unwrap();

    println!("{}", json_out);

    Ok(())
}
