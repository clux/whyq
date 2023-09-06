[private]
default:
  @just --list --unsorted

fmt:
  cargo fmt

test:
  #!/bin/bash
  set -euo pipefail
  export RUST_LOG=debug
  [[ $(cargo run -- -y '.[2].kind' < test/deploy.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- -y '.[2].kind' test/deploy.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- '.[2].metadata' -c < test/deploy.yaml) = '{"name":"controller"}' ]]
  [[ $(cargo run -- '.[2].metadata' -c test/deploy.yaml) = '{"name":"controller"}' ]]
  cargo run -- -y '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' test/deploy.yaml
  cat test/deploy.yaml | cargo run -- '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].readinessProbe' -c
  cargo run -- '.spec.template.spec.containers[].image' -r < test/grafana.yaml
  cargo test

release:
  cargo release minor --execute
