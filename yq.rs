use anyhow::Result;
use serde_yaml::{self, with::singleton_map_recursive, Deserializer};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::*;

use clap::Parser;
#[derive(Parser, Debug)]
struct Args {
    /// Transcode jq JSON output into YAML and emit it
    #[arg(short = 'e', long, default_value = "false")]
    yaml_output: bool,
    /// Arguments passed to jq
    anonymous: Vec<String>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("args: {:?}", args);
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    // pass on args, skip arg 0 (which is yq)
    let yaml_roundtrip = std::env::args().any(|x| x == "-y");
    let mut args = std::env::args().skip(1).filter(|x| x != "-y").collect::<Vec<_>>();
    // read file input either from file or stdin
    let input = read_input_yaml(&mut args).await?;
    let stdout = shellout(input, &args).await?;

    // print output either as yaml or json (as per jq output)
    let output = if yaml_roundtrip {
        let val: serde_json::Value = serde_json::from_slice(&stdout)?;
        serde_yaml::to_string(&val)?
    } else {
        String::from_utf8_lossy(&stdout).to_string()
    };

    println!("{}", output);
    Ok(())
}

/// Pass json encoded bytes to jq with arguments for jq
async fn shellout(input: Vec<u8>, args: &[String]) -> Result<Vec<u8>> {
    debug!("args: {:?}", args);
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
    let stdout = child.wait_with_output().await?.stdout;
    Ok(stdout)
}

/// Convert yaml input into vector of json encoded bytes
async fn read_input_yaml(args: &mut Vec<String>) -> Result<Vec<u8>> {
    let contents; // long lived scope for file case
    let yaml_de = if let Some(last) = args.clone().last() {
        if let Ok(true) = tokio::fs::try_exists(last).await {
            args.pop(); // don't pass the file arg to jq if we read the file
            contents = tokio::fs::read_to_string(last).await?;
            Deserializer::from_str(&contents)
        } else {
            Deserializer::from_reader(std::io::stdin())
        }
    } else {
        Deserializer::from_reader(std::io::stdin())
    };

    let mut docs: Vec<serde_json::Value> = vec![];
    for doc in yaml_de {
        docs.push(singleton_map_recursive::deserialize(doc)?);
    }
    let ser = serde_json::to_vec(&docs)?;
    debug!("decoded json: {}", String::from_utf8_lossy(&ser));
    Ok(ser)
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn stdin() -> Result<()> {
        let data = read_input_yaml(&mut vec!["./test.yaml".into()]).await?;
        //let res = shellout(data.clone(), &[".[2].metadata".into(), "-c".into()]).await?;
        //assert_eq!(String::from_utf8(res)?, "{\"name\":\"version\"}\n".to_string());
        let res = shellout(data, &[".[2].metadata".into(), "-y".into()]).await?;
        assert_eq!(String::from_utf8(res)?, "name: version\n".to_string());
        Ok(())
    }
}
