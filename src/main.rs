use serde_yaml;
use serde_json;
use std::io::{self, BufRead};
use anyhow::{Result, Error};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let yaml_de = serde_yaml::Deserializer::from_reader(stdin);
    let value = serde_yaml::Value::deserialize(yaml_de)?;
    let json = serde_json::from_reader(value)?;
    let ser = serde_json::to_string(json)?;
    println!("{}", ser);
    Ok(())
}
