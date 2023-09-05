# yq
> yet another jq wrapper

A lightweight and portable Rust implementation of the common [jq](https://jqlang.github.io/jq/) wrapper; **`yq`** for doing arbitrary `jq` style queries on YAML documents.

## Features

- arbitrary `jq` usage on yaml input with same syntax (we pass on most args to `jq`)
- handle multidoc yaml input (vector of documents returned)
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

## Installation

```sh
cargo install yjq
```

**Note**: Depends on `jq` being installed.

## Limitations

Only YAML/TOML is supported (no XML - PRs welcome). Shells out to `jq`. Does not preserve [YAML tags](https://yaml.org/spec/1.2-old/spec.html#id2764295) (input is [singleton mapped](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map/index.html) [recursively](https://docs.rs/serde_yaml/latest/serde_yaml/with/singleton_map_recursive/index.html)). No binary builds on CI yet.
