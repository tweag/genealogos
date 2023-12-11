mod nixtract;

use std::io;
use std::io::Read;

fn main() -> Result<(), io::Error> {
    let mut stdin = std::io::stdin();
    let mut buffer = String::new();

    stdin.read_to_string(&mut buffer)?;

    let deserialized: nixtract::NixtractEntry = serde_json::from_str(&buffer).unwrap();

    println!("{:#?}", deserialized);

    Ok(())
}
