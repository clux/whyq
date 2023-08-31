# yq
> yet another jq wrapper

Born out of dissatisfaction with [python yq](https://github.com/kislyuk/yq) (causes huge containers due to python requirement), and [go yq](https://github.com/mikefarah/yq) (uses flags not compatible with jq).

This rust implementation is basically a re-implementation of the python version; it shells out to `jq` after first converting the input to `json`

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
