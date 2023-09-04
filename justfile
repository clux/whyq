[private]
default:
  @just --list --unsorted

test:
  #!/bin/bash
  set -euo pipefail
  [[ $(cargo run -- -y '.[2].kind' < test/version.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- -y '.[2].kind' test/version.yaml) = "ClusterRoleBinding" ]]
  [[ $(cargo run -- '.[2].metadata' -c < test/version.yaml) = '{"name":"version"}' ]]
  [[ $(cargo run -- '.[2].metadata' -c test/version.yaml) = '{"name":"version"}' ]]
  cargo run -- -y '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' test/version.yaml
  cat test/version.yaml | cargo run -- '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].readinessProbe' -c
  cargo run -- '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[].image' -r < test/grafana.yaml
  cargo test

release:
  cargo release minor --execute
