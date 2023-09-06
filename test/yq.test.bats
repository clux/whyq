#!/usr/bin/env bats

@test "stdin_or_file" {
  run yq -y '.[2].kind' < test/deploy.yaml
  echo "$output" && echo "$output" | grep "ClusterRoleBinding"

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
