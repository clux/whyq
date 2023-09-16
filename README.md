# whyq - low overhead yq implementation
[![CI](https://github.com/clux/yq/actions/workflows/release.yml/badge.svg)](https://github.com/clux/yq/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/whyq.svg)](https://crates.io/crates/whyq)

A lightweight and portable [jq](https://jqlang.github.io/jq/) wrapper for doing arbitrary queries from **YAML**/**TOML** documents by converting to **JSON** and passing to `jq`, then returning the result either as raw `jq` output, or back into TOML or YAML.

## Installation

Via cargo:

```sh
cargo install whyq
```

or download a prebuilt from [releases](https://github.com/clux/yq/releases) either manually, or via [binstall](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall whyq
```

**Note**: Depends on `jq` being installed.

## Features

- arbitrary `jq` usage on yaml/toml input with same syntax (args passed along to `jq` with json converted input)
- drop-in replacement to [python-yq](https://kislyuk.github.io/yq/) (e.g. provides: yq)
- handles multidoc **yaml** input (vector of documents returned when multiple docs found)
- handles [yaml merge keys](https://yaml.org/type/merge.html) and expands yaml tags (via `serde_yaml`)
- handles **toml** input (from [Table](https://docs.rs/toml/latest/toml/#parsing-toml))
- allows converting `jq` output to YAML (`-y`) or TOML (`-t`)
- <1MB in binary size (for your small cloud CI images)

## YAML Input
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

Note that YAML is the assumed default input format and primary usage case (and what the binary is named after).

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

Add `alias tq='yq -i=toml'` to your `.bashrc` or `.zshrc` (etc) to make this permanent if you find it useful.

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
All arguments except output selectors such as `-y` or `-t`, and the input arg `-i` are passed on to `jq`.

**Convention**; put `yq` arguments at the front, and `jq` arguments at the back. If it complains put a `--` to separate the argument groups.

**Explaination**: arg parsers are generally struggling with positional leftover arguments containing flags because they lack concepts of "our flags" and "their flags" and will try to match them together. This means combining yq and jq flags into a single arg will not work, and why a convention to explicitly separate the two args exists. Normally the separation is inferred automatically if you put a normal jq query in the middle, but if you don't have any normal positional value arg, you can put a `--` trailing vararg delimiter to indicate that all remaining flags are for `jq`;

```sh
yq -y -c '.[3].kind' < test/deploy.yaml # fails; implicit separation is not detected for a flag first
yq -y '.[3].kind' -c < test/deploy.yaml # works; implicit separation detected after positional
yq -yc '.[3].kind' < test/deploy.yaml # fails; cannot combine of yq and jq args
yq -y -- -c '.[3].kind' < test/deploy.yaml # works; explicit separation
```

## Output Caveats

Output formatting such as `-y` for YAML or `-t` for TOML will require the output from `jq` to be parseable json.
If you pass on `-r` for raw output, then this will not be parseable as json.


## Limitations

- Only YAML/TOML input/output is supported (no XML).
- Shells out to `jq` (only supports what your jq version supports).
- Does not provide rich `-h` or `--help` output (assumes you can use `jq --help` or `man jq`).
- Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html) and then [apply_merged](https://docs.rs/serde_yaml/latest/serde_yaml/value/enum.Value.html#method.apply_merge) before `jq`)
- Does [not support duplicate keys](https://github.com/clux/whyq/issues/14) in the input document
