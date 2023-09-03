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
    #[arg(short = 'y', long, default_value = "false")]
    yaml_output: bool,
    /// Arguments passed to jq (last one might be pre-read if it's a file)
    extra: Vec<String>,
}

impl Args {
    async fn read_input(&mut self) -> Result<Vec<u8>> {
        let contents;
        let yaml_de;
        if let Some(last) = self.extra.clone().last() {
            if let Ok(true) = tokio::fs::try_exists(last).await {
                self.extra.pop(); // don't pass ifle arg to jq - we read
                contents = tokio::fs::read_to_string(last).await?;
                yaml_de = Deserializer::from_str(&contents);
            } else {
                yaml_de = Deserializer::from_reader(std::io::stdin());
            }
        } else {
            yaml_de = Deserializer::from_reader(std::io::stdin());
        };
        let mut docs: Vec<serde_json::Value> = vec![];
        for doc in yaml_de {
            docs.push(singleton_map_recursive::deserialize(doc)?);
        }
        let ser = serde_json::to_vec(&docs)?;
        debug!("decoded json: {}", String::from_utf8_lossy(&ser));
        Ok(ser)
    }

    /// Pass json encoded bytes to jq with arguments for jq
    async fn shellout(&self, input: Vec<u8>) -> Result<Vec<u8>> {
        debug!("jq args: {:?}", self.extra);
        // shellout jq with given args
        let mut child = Command::new("jq")
            .args(&self.extra)
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

    // print output either as yaml or json (as per jq output)
    fn output(&self, stdout: Vec<u8>) -> Result<String> {
        if self.yaml_output {
            let val: serde_json::Value = serde_json::from_slice(&stdout)?;
            Ok(serde_yaml::to_string(&val)?)
        } else {
            Ok(String::from_utf8_lossy(&stdout).to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let mut args = Args::parse();
    debug!("args: {:?}", args);
    let input = args.read_input().await?;
    let stdout = args.shellout(input).await?;
    let output = args.output(stdout)?;
    println!("{}", output);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    impl Args {
        fn new(yaml: bool, args: &[&str]) -> Self {
            Self {
                yaml_output: yaml,
                extra: args.into_iter().map(|x| x.to_string()).collect(),
            }
        }
    }
    #[tokio::test]
    async fn file_input_both_outputs() -> Result<()> {
        let mut args = Args::new(false, &[".[2].metadata", "-c", "test.yaml"]);
        let data = args.read_input().await?;
        let res = args.shellout(data.clone()).await?;
        let out = args.output(res)?;
        assert_eq!(out, "{\"name\":\"version\"}\n");
        args.yaml_output = true;
        let res2 = args.shellout(data).await?;
        let out2 = args.output(res2)?;
        assert_eq!(out2, "name: version\n");
        Ok(())
    }
}
