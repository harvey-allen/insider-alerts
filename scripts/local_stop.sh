#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${CLUSTER_NAME:-insider-alerts}"
KIND_CONTEXT="kind-${CLUSTER_NAME}"

if ! command -v kind >/dev/null 2>&1; then
  echo "kind is not installed; nothing to stop."
  exit 0
fi

if kind get clusters | grep -qx "$CLUSTER_NAME"; then
  if command -v kubectl >/dev/null 2>&1; then
    kubectl config current-context >/dev/null 2>&1 || true
    if kubectl config get-contexts "$KIND_CONTEXT" >/dev/null 2>&1; then
      kubectl config use-context "$KIND_CONTEXT" >/dev/null 2>&1 || true
    fi
  fi

  echo "Deleting kind cluster: $CLUSTER_NAME"
  kind delete cluster --name "$CLUSTER_NAME"
else
  echo "No kind cluster named $CLUSTER_NAME found."
fi