use anyhow::{Error, Result};
use serde_json::{self, Value};
use serde_yaml::{self, with::singleton_map_recursive as smr};

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let yaml_de = serde_yaml::Deserializer::from_reader(stdin);
    let mut docs: Vec<Value> = vec![];
    for doc in yaml_de {
        docs.push(smr::deserialize(doc)?);
    }
    let ser = serde_json::to_string(&docs)?;
    println!("{}", ser);
    Ok(())
}
