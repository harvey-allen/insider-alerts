#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K8S_DIR="$ROOT_DIR/k8s"
CLUSTER_NAME="${CLUSTER_NAME:-insider-alerts}"
NAMESPACE="${NAMESPACE:-insider-alerts}"
IMAGE_NAME="${IMAGE_NAME:-insider-alerts:local}"
KIND_CONTEXT="kind-${CLUSTER_NAME}"

require_command() {
  local command_name="$1"
  local install_hint="$2"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name"
    echo "$install_hint"
    exit 1
  fi
}

ensure_kind() {
  if command -v kind >/dev/null 2>&1; then
    return
  fi

  if command -v brew >/dev/null 2>&1; then
    echo "kind not found. Installing with Homebrew..."
    brew install kind
    return
  fi

  echo "kind is required but not installed. Install it and rerun this script."
  exit 1
}

ensure_docker_running() {
  local attempt

  if docker info >/dev/null 2>&1; then
    return
  fi

  if [[ "$(uname -s)" == "Darwin" ]] && command -v open >/dev/null 2>&1; then
    echo "Docker daemon is not running. Launching Docker Desktop..."
    open -a Docker || true

    for attempt in $(seq 1 24); do
      if docker info >/dev/null 2>&1; then
        echo "Docker Desktop is ready."
        return
      fi

      sleep 5
    done
  fi

  echo "Docker is installed but the daemon is not running. Start Docker and rerun this script."
  exit 1
}

wait_for_redis_data() {
  local redis_pod
  local attempt
  local keys

  redis_pod="$(kubectl -n "$NAMESPACE" get pods -l app=redis -o jsonpath='{.items[0].metadata.name}')"

  echo "Waiting for Redis to receive monitor data..."
  for attempt in $(seq 1 18); do
    keys="$(kubectl -n "$NAMESPACE" exec "$redis_pod" -- redis-cli --raw KEYS 'monitor:*' | tr -d '\r')"
    if [[ -n "$keys" ]]; then
      echo "Redis keys detected:"
      printf '%s\n' "$keys"
      return
    fi

    sleep 5
  done

  echo "No monitor data found in Redis yet. The worker may still be polling upstream feeds."
}

require_command docker "Install Docker Desktop and ensure the daemon is running."
require_command kubectl "Install kubectl before running this script."
ensure_kind

echo "Checking Docker daemon..."
ensure_docker_running

if ! kind get clusters | grep -qx "$CLUSTER_NAME"; then
  echo "Creating kind cluster: $CLUSTER_NAME"
  kind create cluster --name "$CLUSTER_NAME"
else
  echo "Using existing kind cluster: $CLUSTER_NAME"
fi

kubectl config use-context "$KIND_CONTEXT" >/dev/null

echo "Building Docker image: $IMAGE_NAME"
docker build -t "$IMAGE_NAME" "$ROOT_DIR"

echo "Loading image into kind cluster"
kind load docker-image "$IMAGE_NAME" --name "$CLUSTER_NAME"

echo "Applying Kubernetes manifests"
kubectl apply -f "$K8S_DIR/00-namespace.yaml"
kubectl apply -f "$K8S_DIR/10-app-config.yaml"
kubectl apply -f "$K8S_DIR/20-redis.yaml"

echo "Waiting for Redis StatefulSet rollout"
kubectl -n "$NAMESPACE" rollout status statefulset/redis --timeout=180s

echo "Deploying application"
sed "s|your-registry/insider-alerts:latest|$IMAGE_NAME|g" "$K8S_DIR/30-app.yaml" | kubectl apply -f -

echo "Waiting for app Deployment rollout"
kubectl -n "$NAMESPACE" rollout status deployment/insider-alerts-app --timeout=180s

echo "Current pod status"
kubectl -n "$NAMESPACE" get pods -o wide

echo "Verifying Redis connectivity"
kubectl -n "$NAMESPACE" exec statefulset/redis -- redis-cli ping

wait_for_redis_data

echo "Recent app logs"
kubectl -n "$NAMESPACE" logs deployment/insider-alerts-app --tail=40

echo
echo "Local Kubernetes setup is ready."
echo "Useful commands:"
echo "  kubectl -n $NAMESPACE get pods"
echo "  kubectl -n $NAMESPACE logs -f deployment/insider-alerts-app"
echo "  kubectl -n $NAMESPACE exec statefulset/redis -- redis-cli LLEN monitor:SEC:transactions"