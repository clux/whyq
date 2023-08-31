[private]
default:
  @just --list --unsorted

test:
  #!/bin/bash
  set -euxo pipefail
  [[ $(cargo run -- '.[2].kind' -y < test.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- '.[2].kind' -y test.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- '.[2].metadata' -c < test.yaml) = '{"name":"version"}' ]]
  [[ $(cargo run -- '.[2].metadata' -c test.yaml) = '{"name":"version"}' ]]
  cargo test

release:
  cargo release minor --execute
