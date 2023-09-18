# whyq - low overhead yq implementation
[![CI](https://github.com/clux/yq/actions/workflows/release.yml/badge.svg)](https://github.com/clux/yq/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/whyq.svg)](https://crates.io/crates/whyq)

A lightweight and portable [jq](https://jqlang.github.io/jq/) wrapper for doing arbitrary queries from **YAML**/**TOML**/**JSON** documents by converting to **JSON** and passing to `jq`, then returning the result either as raw `jq` output, or back into TOML or YAML.

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

## Why / Why Not

### jq compatibility

- arbitrary `jq` usage on any input format (yaml/toml/json)
- [same filter syntax](https://jqlang.github.io/jq/manual/#basic-filters)
- matches `jq`'s cli interface (only some extra input/output format controlling flags)
- supports `jq` output formatters such as `-c`, `-r`, and `-j` (compact, raw, joined output resp)
- supports [jq modules](https://jqlang.github.io/jq/manual/#modules) on all input formats

### Features

- reads __multidoc yaml__ input, handles [yaml merge keys](https://yaml.org/type/merge.html) (expanding tags)
- reads from __stdin xor file__ (file if last arg is a file)
- output conversion shortcuts: `-y` (YAML) or `-t` (TOML)
- ~1MB in binary size (for small cloud CI images / [binstalled ci actions](https://github.com/cargo-bins/cargo-binstall#faq))
- drop-in replacement to [python-yq](https://kislyuk.github.io/yq/) (`provides: yq`)

### Limitations

- Shells out to `jq` (supports what your `jq` version supports)
- Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html) and then [apply_merged](https://docs.rs/serde_yaml/latest/serde_yaml/value/enum.Value.html#method.apply_merge) before `jq`)
- Does not __preserve indentation__ (unsupported in [serde_yaml](https://github.com/dtolnay/serde-yaml/issues/337))
- Does not support [duplicate keys](https://github.com/clux/whyq/issues/14) in the input document
- No XML/CSV support (or other more exotic formats)

## Usage
### YAML Input

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

The default input format is YAML and is what the binary is named for (and the most common primary usage case).

### TOML Input

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

Add `alias tq='yq --input=toml'` to your `.bashrc` or `.zshrc` (etc) to make this permanent if you find it useful.

### JSON Input

If you need to convert json to another format you pass `--input=json`:

```sh
$ yq --input=json '.ingredients | keys' -y < test/guacamole.json                                                                          ☸ production-eu-west-1󰛢monitoring
- avocado
- coriander
- cumin
- garlic
- lime
- onions
- pepper
- salt
- tomatoes
```

### Advanced Examples
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

Using helpers from `jq` [modules](https://jqlang.github.io/jq/manual/):

```sh
$ yq 'include "k"; .[] | gvk' -r -L$PWD/test/modules < test/deploy.yaml
v1.ServiceAccount
rbac.authorization.k8s.io/v1.ClusterRole
rbac.authorization.k8s.io/v1.ClusterRoleBinding
v1.Service
apps/v1.Deployment
```

### Output Caveats

Output formatting such as `-y` for YAML or `-t` for TOML will require the output from `jq` to be parseable json.
If you pass on `-r`,`-c` or `-c` for raw/compact output, then this will generally not be parseable as json.
