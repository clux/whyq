use anyhow::Result;
use serde_json::{self, Value};
use serde_yaml::{self, with::singleton_map_recursive as smr};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let stdin = std::io::stdin();
    // TODO: read from file arg if no stdin
    let yaml_de = serde_yaml::Deserializer::from_reader(stdin);
    let mut docs: Vec<Value> = vec![];
    for doc in yaml_de {
        docs.push(smr::deserialize(doc)?);
    }
    let ser = serde_json::to_vec(&docs)?;
    debug!("decoded json: {}", String::from_utf8_lossy(&ser));
    // pass on args, skip arg 0 (which is yq)
    let mut all_args = std::env::args().into_iter().skip(1).collect::<Vec<_>>();
    let yaml_output = all_args.contains(&"-y".to_string());
    let args = all_args
        .into_iter()
        .filter(|x| x != "-y")
        .collect::<Vec<_>>();
    // TODO: slice off -y arg
    debug!("args: {:?}", args);
    let mut child = Command::new("jq")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&ser).await.unwrap();
    drop(stdin);
    let output = child.wait_with_output().await?;
    let stdout = output.stdout;
    if yaml_output {
        let val: serde_json::Value = serde_json::from_slice(&stdout)?;
        let ser2 = serde_yaml::to_string(&val)?;
        println!("{}", ser2);
    } else {
        println!("{}", String::from_utf8_lossy(&stdout));
    }
    Ok(())
}
