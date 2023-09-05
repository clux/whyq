# yq
> yet another jq wrapper

A lightweight and portable Rust implementation of the common [jq](https://jqlang.github.io/jq/) wrapper; **`yq`** for doing arbitrary `jq` style queries on YAML documents.

## Features

- arbitrary `jq` usage on yaml input with same syntax (we pass on most args to `jq`)
- handle multidoc yaml input (vector of documents returned)
- unpack yaml tags (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html))
- allows converting `jq` output back to YAML with `-y`

## Usage
Use as [jq](https://jqlang.github.io/jq/tutorial/) either via stdin:

```sh
$ yq '.[3].kind' -r < test/version.yaml
Service

$ yq -y '.[3].metadata' < test/grafana.yaml
labels:
  app: grafana
name: grafana
namespace: monitoring
```

or from a file arg (at the end):

```sh
$ yq '.[3].kind' -- -r test/version.yaml
$ yq -y '.[3].metadata' test/version.yaml
```

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
$ yq -y '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' test/version.yaml
8000
```

## Argument Priority
All arguments except `-y` / `--yaml-output` is passed on to `jq`.
Sometimes `clap` gets confused about positional arguments vs options when you put a flag before the query. To explicitly specify where `jq` args begin add a `--` delimiters to the flag;

```sh
yq -y -c '.[3].kind' < test/version.yaml # inference fails
yq -y '.[3].kind' -c < test/version.yaml # works
yq -yc '.[3].kind' < test/version.yaml # unsupported combining of yq and jq args
yq -y -- -c '.[3].kind' < test/version.yaml # works; explicit separation
```

## Output Caveats

Output formatting such as `-y` for YAML will require the output from `jq` to be parseable json. If you pass on `-r` for raw output, then this will not be parseable as json.

## Installation

```sh
cargo install yjq
```

**Note**: Depends on `jq` being installed.

## Limitations

Only YAML is supported (no TOML / XML - PRs welcome). Shells out to `jq`. Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html)). No binary builds on CI yet.
