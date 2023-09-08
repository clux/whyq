# whyq - low overhead yq implementation
[![CI](https://github.com/clux/yq/actions/workflows/release.yml/badge.svg)](https://github.com/clux/yq/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/whyq.svg)](https://crates.io/crates/whyq)

A lightweight and portable [jq](https://jqlang.github.io/jq/) wrapper written in Rust for doing arbitrary `jq` queries on **YAML** or **TOML** documents by transcoding through **JSON**.
## Installation

Via cargo:

```sh
cargo install whyq
```

or download a prebuilt from [releases](https://github.com/clux/yq/releases) either manually, or via [binstall](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall whyq
```

**Note**: Depends on `jq` being installed, and `provides` the `yq` binary.

## Features

- arbitrary `jq` usage on yaml/toml input with same syntax (we pass on args to `jq` along with json converted input)
- generally functions as a drop-in replacement to [python-yq](https://kislyuk.github.io/yq/) (e.g. provides: yq)
- handles multidoc **yaml** input (vector of documents returned when multiple docs found)
- handles **toml** input (`toml::Table` -> json)
- unpack yaml tags (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html))
- allows converting `jq` output to YAML (`-y`) or TOML (`-t`)

## Usage
Use as [jq](https://jqlang.github.io/jq/tutorial/) either via stdin:

```sh
$ yq '.[3].kind' -r < test/deploy.yaml
Service

$ yq -y '.[3].metadata' < test/deploy.yaml
labels:
  app: controller
name: controller
namespace: default
```

or from a file arg (at the end):

```sh
$ yq '.[3].kind' -r test/deploy.yaml
$ yq -y '.[3].metadata' test/deploy.yaml
```

Stdin is always used if it's piped to.

## TOML Input
Using say `Cargo.toml` from this repo as input, and aliasing `tq='yq -i=toml'`:

```sh
$ tq '.package.categories[]' -r < Cargo.toml
command-line-utilities
parsing

$ tq -t '.package.metadata' < Cargo.toml
[binstall]
bin-dir = "yq-{ target }/{ bin }{ format }"
pkg-url = "{ repo }/releases/download/{ version }/yq-{ target }{ archive-suffix }"

$ tq -y '.dependencies.clap' < Cargo.toml
features:
- cargo
- derive
version: 4.4.2

$ tq '.profile' -c < Cargo.toml
{"release":{"lto":true,"panic":"abort","strip":"symbols"}}
```

Add `alias tq='yq -i=toml'` to your `.bashrc` or `.zshrc` to make this permanent if you find it useful.

## Advanced Examples
Select with nested query and raw output:

```sh
$ yq '.spec.template.spec.containers[].image' -r < test/grafana.yaml
quay.io/kiwigrid/k8s-sidecar:1.24.6
quay.io/kiwigrid/k8s-sidecar:1.24.6
docker.io/grafana/grafana:10.1.0
```

Select on multidoc:

```sh
$ yq -y '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' test/deploy.yaml
8000
```

Escaping keys with slashes etc in them:

```sh
yq -y '.updates[] | select(.["package-ecosystem"] == "cargo") | .groups' .github/dependabot.yml
```

## Argument Priority
All arguments except output selectors such as `-y` or `-t` are passed on to `jq`.

**Convention**; put `yq` arguments at the front, and `jq` arguments at the back. If it complains put a `--` to separate the argument groups.

**Explaination**: arg parsers are generally struggling with positional leftover arguments containing flags because they lack concepts of "our flags" and "their flags" and will try to match them together. This means combining yq and jq flags into a single arg will not work, and why a convention to explicitly separate the two args exists. Normally the separation is inferred automatically if you put a normal jq query in the middle, but if you don't have any normal positional value arg, you can put a `--` trailing vararg delimiter to indicate that all remaining flags are for `jq`;

```sh
yq -y -c '.[3].kind' < test/deploy.yaml # fails; implicit separation is not detected for a flag first
yq -y '.[3].kind' -c < test/deploy.yaml # works; implicit separation detected after positional
yq -yc '.[3].kind' < test/deploy.yaml # fails; cannot combine of yq and jq args
yq -y -- -c '.[3].kind' < test/deploy.yaml # works; explicit separation
```

## Output Caveats

Output formatting such as `-y` for YAML or `-t` for TOML will require the output from `jq` to be parseable json. If you pass on `-r` for raw output, then this will not be parseable as json.


## Limitations

Only YAML/TOML is supported. Shells out to `jq` (only supports what your jq version supports).
Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html)).
