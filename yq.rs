use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde_yaml::{self, with::singleton_map_recursive, Deserializer};
use std::io::{stderr, stdin, BufReader, IsTerminal, Read, Write};
use std::process::{Command, Stdio};
use tracing::*;

#[derive(Copy, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Input {
    #[default]
    Yaml,
    Toml,
}

/// A lightweight and portable Rust implementation of a common jq wrapper
///
/// Allows doing arbitrary jq style queries editing on YAML documents.
///
/// yq '.[3].kind' < .github/dependabot.yaml
///
/// yq -y '.updates[0].schedule' .github/dependabot.yml
///
/// yq '.spec.template.spec.containers[].image' -r
///
/// yq '.[].kind' -r < manifest.yml
///
/// yq -y '.[2].metadata' < manifest.yml
#[derive(Parser, Debug, Default)]
#[command(author, version, about)]
struct Args {
    /// Transcode jq JSON output into YAML and emit it
    #[arg(short = 'y', long, default_value = "false", conflicts_with = "toml_output")]
    yaml_output: bool,
    /// Transcode jq JSON output into TOML and emit it
    #[arg(short = 't', long, default_value = "false", conflicts_with = "yaml_output")]
    toml_output: bool,
    /// Input format
    #[arg(long, value_enum, default_value_t)]
    input: Input,

    /// Inplace editing
    #[arg(short, long, default_value = "false", requires = "file")]
    in_place: bool,

    #[arg(default_value = "false")]
    file: Option<PathBuf>,

    // ----- jq arguments
    /// Output strings without escapes and quotes
    #[arg(short, long, default_value = "false")]
    raw_output: bool,

    /// Compact instead of pretty-printed output
    ///
    /// This is unlikely to work with yaml or toml output because it requires
    /// that the jq -c output is deserializable into the desired output format.
    #[arg(short = 'c', long, default_value = "false")]
    compact_output: bool,

    /// Output strings without escapes and quotes
    ///
    /// This is unlikely to work with yaml or toml output because it requires
    /// that the jq -r output is deserializable into the desired output format.
    #[arg(short = 'r', long, default_value = "false")]
    raw_output: bool,

    /// Output strings without escapes and quotes, without newlines after each output
    ///
    /// This is unlikely to work with yaml or toml output because it requires
    /// that the jq -r output is deserializable into the desired output format.
    #[arg(short = 'c', long, default_value = "false")]
    join_output: bool,
}

impl Args {
    fn read_yaml(&mut self) -> Result<Vec<u8>> {
        let yaml_de = if !stdin().is_terminal() && !cfg!(test) {
            debug!("reading from stdin");
            Deserializer::from_reader(stdin())
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
            let json_value: serde_json::Value = {
                let mut yaml_doc: serde_yaml::Value = singleton_map_recursive::deserialize(doc)?;
                yaml_doc.apply_merge()?;
                let yaml_ser = serde_yaml::to_string(&yaml_doc)?;
                serde_yaml::from_str(&yaml_ser)?
            };
            docs.push(json_value);
        }
        debug!("found {} documents", docs.len());
        // if there is 1 or 0 documents, do not return as nested documents
        let ser = match docs.as_slice() {
            [x] => serde_json::to_vec(x)?,
            [] => serde_json::to_vec(&serde_json::json!({}))?,
            xs => serde_json::to_vec(xs)?,
        };
        Ok(ser)
    }
    fn read_toml(&mut self) -> Result<Vec<u8>> {
        use toml::Table;
        let mut buf = String::new();
        let toml_str = if !stdin().is_terminal() && !cfg!(test) {
            debug!("reading from stdin");
            stdin().read_to_string(&mut buf)?;
            buf
        } else if let Some(f) = self.extra.pop() {
            if !std::path::Path::new(&f).exists() {
                Self::try_parse_from(["cmd", "-h"])?;
                std::process::exit(2);
            }
            std::fs::read_to_string(f)?
        } else {
            Self::try_parse_from(["cmd", "-h"])?;
            std::process::exit(2);
        };
        let doc: Table = toml_str.parse()?;
        let doc_as: serde_json::Value = doc.try_into()?;
        Ok(serde_json::to_vec(&doc_as)?)
    }
    fn read_input(&mut self) -> Result<Vec<u8>> {
        let ser = match self.input {
            Input::Yaml => self.read_yaml()?,
            Input::Toml => self.read_toml()?,
        };
        debug!("input decoded as json: {}", String::from_utf8_lossy(&ser));
        Ok(ser)
    }

    /// Pass json encoded bytes to jq with arguments for jq
    fn shellout(&self, input: Vec<u8>) -> Result<Vec<u8>> {
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
        stdin.write_all(&input).unwrap();
        drop(stdin);
        // then wait for exit and gather output
        let output = child.wait_with_output()?;
        if !output.status.success() {
            anyhow::bail!("arguments rejected by jq: {}", output.status);
        }
        debug!("jq stdout: {}", String::from_utf8_lossy(&output.stdout));
        Ok(output.stdout)
    }

    // print output either as yaml or json (as per jq output)
    fn output(&self, stdout: Vec<u8>) -> Result<String> {
        if self.yaml_output {
            // NB: this can fail - particularly if people use -r or work on multidoc
            let val: serde_json::Value = serde_json::from_slice(&stdout)?;
            let data = serde_yaml::to_string(&val)?.trim_end().to_string();
            Ok(data)
        } else if self.toml_output {
            let val: serde_json::Value = serde_json::from_slice(&stdout)?;
            Ok(toml::to_string(&val)?.trim_end().to_string())
        } else {
            // NB: stdout here is not always json - users can pass -r to jq
            Ok(String::from_utf8_lossy(&stdout).trim_end().to_string())
        }
    }
}

fn init_env_tracing_stderr() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
    let logger = tracing_subscriber::fmt::layer().compact().with_writer(stderr);
    let env_filter = EnvFilter::try_from_default_env().or(EnvFilter::try_new("info"))?;
    let collector = Registry::default().with(logger).with(env_filter);
    Ok(tracing::subscriber::set_global_default(collector)?)
}

fn main() -> Result<()> {
    init_env_tracing_stderr()?;
    let mut args = Args::parse();
    debug!("args: {:?}", args);
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
                toml_output: false,
                input: Input::Yaml,
                extra: args.into_iter().map(|x| x.to_string()).collect(),
            }
        }
    }
    #[test]
    fn file_input_both_outputs() -> Result<()> {
        init_env_tracing_stderr()?;
        let mut args = Args::new(false, &[".[2].metadata", "-c", "test/deploy.yaml"]);
        println!("have stdin? {}", !std::io::stdin().is_terminal());
        let data = args.read_input().unwrap();
        println!("debug args: {:?}", args);
        let res = args.shellout(data.clone()).unwrap();
        let out = args.output(res)?;
        assert_eq!(out, "{\"name\":\"controller\"}");
        args.yaml_output = true;
        let res2 = args.shellout(data)?;
        let out2 = args.output(res2)?;
        assert_eq!(out2, "name: controller");
        Ok(())
    }
}
