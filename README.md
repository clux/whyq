# yq
> yet another jq wrapper

A Rust implementation of the common [jq](https://jqlang.github.io/jq/) wrapper; **`yq`** (not a `jq` implementation, as it depends on `jq`).

Born out of dissatisfaction with the existing solutions. The best current alternative is [python-yq](https://github.com/kislyuk/yq) as it preserves `jq` arguments (making it easy to learn and use), but causes huge docker containers that make it unsuitable on CI. This implementation tries to replace the functionality of the python version.

## Usage
Supports any query functionality [supported by jq](https://jqlang.github.io/jq/tutorial/) either via stdin:

```sh
$ yq '.[3].kind' -r < deployment.yaml
Service

$ yq '.[3].metadata' -y < deployment.yaml
labels:
  app: version
name: version
namespace: default
```

or from a file arg (at the end):

```sh
$ yq '.[3].kind' -r deployment.yaml
$ yq '.[3].metadata' -y deployment.yaml
```

## Installation

```sh
cargo install yjq
```

**NB**: Depends on `jq` being installed.
