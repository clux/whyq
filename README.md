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

Using say `Cargo.toml` from this repo as input, and aliasing `tq='yq --input=toml'`:

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

## Output Caveats

Output formatting such as `-y` for YAML or `-t` for TOML will require the output from `jq` to be parseable json.
If you pass on `-r` for raw output, then this will not be parseable as json.


## Limitations

- Only YAML/TOML input/output is supported (no XML).
- Shells out to `jq` (only supports what your jq version supports).
- Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html) and then [apply_merged](https://docs.rs/serde_yaml/latest/serde_yaml/value/enum.Value.html#method.apply_merge) before `jq`)
- Does not preserve (or allow customizing) indentation in the output (supported in [serde_json](https://docs.rs/serde_json/latest/serde_json/ser/struct.PrettyFormatter.html), but unsupported in [serde_yaml](https://github.com/dtolnay/serde-yaml/issues/337))
- Does [not support duplicate keys](https://github.com/clux/whyq/issues/14) in the input document
