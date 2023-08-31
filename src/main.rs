use anyhow::{bail, Error, Result};
use serde_json::{self, Value};
use serde_yaml::{self, with::singleton_map_recursive as smr};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::{ChildStdin, Command};
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
    let args = std::env::args().into_iter().skip(1).collect::<Vec<_>>();
    // TODO: slice off -y arg
    debug!("args: {:?}", args);
    let mut cmd = Command::new("jq")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdin = cmd.stdin.take().expect("Failed to open stdin");
    stdin
        .write_all(&ser)
        .await
        .expect("Failed to write to stdin");
    stdin.write(b"\n").await?;
    stdin.flush().await?;
    //stdin.poll_shutdown().await?;
    //let output = cmd.wait_with_output().await.unwrap();
    //info!("done, {:?}", output);
    //if output.stderr.is_empty() {
    //    info!("ww {}", String::from_utf8_lossy(&output.stdout));
    //    Ok(())
    //} else {
    //    warn!("{:?}", String::from_utf8_lossy(&output.stderr));
    //    Ok(())
    //}
    Ok(())
}
