#!/usr/bin/env bats

@test "stdin" {
  run yq -y '.[2].kind' < test/deploy.yaml
  echo "$output" && echo "$output" | grep "ClusterRoleBinding"
}

@test "file" {
  if [[ "${CI}" =~ "true" ]]; then
    skip # isTerminal seems to do the wrong thing on github actions..
  fi
  yq -y '.[2].kind' test/deploy.yaml
  run yq -y '.[2].kind' test/deploy.yaml
  echo "$output" && echo "$output" | grep "ClusterRoleBinding"
}

@test "compact_json_output" {
  run yq '.[2].metadata' -c < test/deploy.yaml
  echo "$output" && echo "$output" | grep '{"name":"controller"}'
}

@test "nested_select" {
  run yq '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].ports[0].containerPort' -r < test/deploy.yaml
  echo "$output" && echo "$output" | grep "8000"
}

@test "nested_select_json" {
  run yq '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].readinessProbe' -c < test/deploy.yaml
  echo "$output" && echo "$output" | grep '{"httpGet":{"path":"/health","port":"http"},"initialDelaySeconds":5,"periodSeconds":5}'

  run yq '.spec.template.spec.containers[].image' -r < test/grafana.yaml
}

@test "jq_compat" {
  cat test/deploy.yaml | yq '.[] | select(.kind == "Deployment") | .spec.template.spec.containers[0].readinessProbe' -c > test/output.json
  run jq ".httpGet.path" test/output.json
  echo "$output" && echo "$output" | grep '"/health"'
  rm test/output.json
}

@test "yq_in_place_edit" {
  cat test/secret.yaml |  yq -i '.metadata.name="updated-name"' > test/output.yaml
  cat test/output.yaml | yq '.metadata.name' | grep 'updated-name'
  rm test/output.yaml
}

@test "exit_codes" {
  run yq -h
  [ "$status" -eq 0 ]
  run yq --help
  [ "$status" -eq 0 ]
  if [[ "${CI}" =~ "true" ]]; then
    skip # ci is fun
  fi
  run yq
  [ "$status" -eq 1 ]
}

@test "toml" {
  run yq --input=toml -y '.package.edition' -r < Cargo.toml
  echo "$output" && echo "$output" | grep '2021'

  run yq --input=toml '.dependencies.clap.features' -c < Cargo.toml
  echo "$output" && echo "$output" | grep '["cargo","derive"]'
}

@test "yaml_merge" {
  run yq '.workflows.my_flow.jobs[0].build' -c < test/circle.yml
  echo "$output" && echo "$output" | grep '{"filters":{"tags":{"only":"/.*/"}}}'

  run yq '.jobs.build.steps[1].run.name' -r < test/circle.yml
  echo "$output" && echo "$output" | grep "Version information"
}

@test "inplace" {
  run yq -yi '.kind = "Hahah"' test/grafana.yaml
  run yq -r .kind test/grafana.yaml
  echo "$output" && echo "$output" | grep "Hahah"
  yq -yi '.kind = "Deployment"' test/grafana.yaml # undo
}

@test "join" {
  run yq -j '.spec.template.spec.containers[].image' test/grafana.yaml
  echo "$output" && echo "$output" | grep "quay.io/kiwigrid/k8s-sidecar:1.24.6quay.io/kiwigrid/k8s-sidecar:1.24.6docker.io/grafana/grafana:10.1.0"
}

@test "json_input" {
  run yq --input=json ".ingredients | keys" -c < test/guacamole.json
  echo "$output" && echo "$output" | grep '["avocado","coriander","cumin","garlic","lime","onions","pepper","salt","tomatoes"]'
}

@test "jq_modules" {
  run yq 'include "k"; . | gvk' -r -L$PWD/test/modules < test/grafana.yaml
  echo "$output" && echo "$output" | grep 'apps/v1.Deployment'
}

@test "paramless" {
  run yq -y <<< '["foo"]'
  echo "$output" && echo "$output" | grep '\- foo'
  run yq <<< '"bar"'
  echo "$output" && echo "$output" | grep '"bar"'
}

@test "multidoc-jq-output-to-yaml" {
  run yq '.[].metadata.labels' -y test/deploy.yaml
  echo "$output" && echo "$output" | rg -U '\- null\n- null\n- null\n- app: controller\n- app: controller'
}
