[private]
default:
  @just --list --unsorted

fmt:
  cargo fmt

test:
  #!/usr/bin/env bash
  set -euo pipefail
  export RUST_LOG=debug
  [[ $(yq -y '.[2].kind' < test/deploy.yaml) = "ClusterRoleBinding" ]]
  [[ $(yq -y '.[2].kind' test/deploy.yaml) = "ClusterRoleBinding" ]]
  [[ $(yq '.[2].metadata' -c < test/deploy.yaml) = '{"name":"controller"}' ]]
  [[ $(yq '.[2].metadata' -c test/deploy.yaml) = '{"name":"controller"}' ]]
  yq -y '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' test/deploy.yaml
  cat test/deploy.yaml | yq '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].readinessProbe' -c
  yq '.spec.template.spec.containers[].image' -r < test/grafana.yaml
  cargo test

release:
  cargo release minor --execute
