mod model;
mod nixtract;

use std::io;
use std::io::Read;

use crate::model::Model;
use crate::nixtract::Nixtract;

fn main() -> Result<(), io::Error> {
    let mut stdin = std::io::stdin();
    let mut buffer = String::new();

    stdin.read_to_string(&mut buffer)?;

    let deserialized: nixtract::NixtractEntry = serde_json::from_str(&buffer).unwrap();
    let nixtract: Nixtract = Nixtract {
        entries: vec![deserialized],
    };

    let model: Model = nixtract.into();

    println!("{:#?}", model);

    Ok(())
}
