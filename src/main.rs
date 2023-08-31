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
    // pass on args, skip arg 0 (which is yq)
    let all_args = std::env::args().into_iter().skip(1).collect::<Vec<_>>();
    let yaml_output = all_args.contains(&"-y".to_string());
    let args = all_args
        .into_iter()
        .filter(|x| x != "-y") // yq only arg
        .collect::<Vec<_>>();
    debug!("args: {:?}", args);

    // read file input either from file or stdin
    let input = read_input_yaml(&args.last()).await?;

    // shellout jq with given args
    let mut child = Command::new("jq")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    // pass file input as stdin
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&input).await.unwrap();
    drop(stdin);
    // then wait for exit and gather output
    let output = child.wait_with_output().await?;
    let stdout = output.stdout;
    // print output either as yaml or json (as per jq output)
    if yaml_output {
        let val: serde_json::Value = serde_json::from_slice(&stdout)?;
        let ser2 = serde_yaml::to_string(&val)?;
        println!("{}", ser2);
    } else {
        println!("{}", String::from_utf8_lossy(&stdout));
    }
    Ok(())
}

/// Convert yaml input into vector of json encoded bytes
async fn read_input_yaml(last_arg: &Option<&String>) -> Result<Vec<u8>> {
    let contents; // long lived scope for file case
    let yaml_de = if let Some(last) = last_arg {
        if let Ok(true) = tokio::fs::try_exists(last).await {
            contents = tokio::fs::read_to_string(last).await?;
            serde_yaml::Deserializer::from_str(&contents)
        } else {
            let stdin = std::io::stdin();
            serde_yaml::Deserializer::from_reader(stdin)
        }
    } else {
        let stdin = std::io::stdin();
        serde_yaml::Deserializer::from_reader(stdin)
    };

    let mut docs: Vec<Value> = vec![];
    for doc in yaml_de {
        docs.push(smr::deserialize(doc)?);
    }
    let ser = serde_json::to_vec(&docs)?;
    debug!("decoded json: {}", String::from_utf8_lossy(&ser));
    Ok(ser)
}
