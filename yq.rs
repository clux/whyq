use anyhow::Result;
use clap::Parser;
use serde_yaml::{self, with::singleton_map_recursive, Deserializer};
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};
use tracing::*;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Transcode jq JSON output into YAML and emit it
    #[arg(short = 'y', long, default_value = "false")]
    yaml_output: bool,
    /// Arguments passed to jq (last one might be pre-read if it's a file)
    #[arg(trailing_var_arg = true)]
    extra: Vec<String>,
}
// PROBLEM1: "-yc" combined flag fails to match flag and pass -c to varargs..
// SOLN1: separate -y and -c with -y going first (don't see any other good solns..)
// PROBLEM2: cannot pass flags before query
// SOLN: pass flags after query
// PROBLEM2 (more general): not clear where our args end and jqs arg start
// SOLN2: allow -- as a trailing_var_arg delimiter to force everything after to jq

impl Args {
    fn read_input(&mut self) -> Result<Vec<u8>> {
        let yaml_de = if !atty::is(atty::Stream::Stdin) {
            Deserializer::from_reader(std::io::stdin())
        } else if let Some(f) = self.extra.pop() {
            if !std::path::Path::new(&f).exists() {
                Self::try_parse_from(["cmd", "-h"])?;
                std::process::exit(2);
            }
            let file = std::fs::File::open(f)?;
            // NB: can do everything async (via tokio + tokio_util) except this:
            // serde only has a sync reader interface, so may as well do all sync.
            Deserializer::from_reader(BufReader::new(file))
        } else {
            Self::try_parse_from(["cmd", "-h"])?;
            std::process::exit(2);
        };

        let mut docs: Vec<serde_json::Value> = vec![];
        for doc in yaml_de {
            docs.push(singleton_map_recursive::deserialize(doc)?);
        }
        // if there is 1 or 0 documents, do not return as nested documents
        let ser = match docs.as_slice() {
            [x] => serde_json::to_vec(x)?,
            [] => serde_json::to_vec(&serde_json::json!({}))?,
            xs => serde_json::to_vec(xs)?,
        };
        debug!("decoded json: {}", String::from_utf8_lossy(&ser));
        Ok(ser)
    }

    /// Pass json encoded bytes to jq with arguments for jq
    fn shellout(&self, input: Vec<u8>) -> Result<Vec<u8>> {
        println!("jq args: {:?}", self.extra);
        // shellout jq with given args
        let mut child = Command::new("jq")
            .args(&self.extra)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        // pass file input as stdin
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&input).unwrap();
        drop(stdin);
        // then wait for exit and gather output
        let output = child.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!("arguments rejected by jq: {}", output.status);
        }
        Ok(output.stdout)
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

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let mut args = Args::try_parse()?;
    println!("args: {:?}", args);
    let input = args.read_input()?;
    let stdout = args.shellout(input)?;
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
    #[test]
    fn file_input_both_outputs() -> Result<()> {
        let mut args = Args::new(false, &[".[2].metadata", "-c", "test/version.yaml"]);
        let data = args.read_input().unwrap();
        let res = args.shellout(data.clone()).unwrap();
        let out = args.output(res)?;
        assert_eq!(out, "{\"name\":\"version\"}\n");
        args.yaml_output = true;
        let res2 = args.shellout(data)?;
        let out2 = args.output(res2)?;
        assert_eq!(out2, "name: version\n");
        Ok(())
    }
}
