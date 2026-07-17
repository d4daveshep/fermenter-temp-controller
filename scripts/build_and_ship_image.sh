#!/usr/bin/env bash
#
# Cross-compiles the fermenter release image for linux/arm64 (per
# rewrite-plan.md §11 / fermenter/Dockerfile), saves it to a tarball, and
# scp's it to a Raspberry Pi. This scripts the manual "docker
# save/scp/docker load" transfer path documented in slice-7-deployment's
# tasks.md (task 5.1) — it does not change how the image is built, how
# Compose orchestrates it, or the deployment-packaging capability's
# requirements; it just automates a step that was previously done by hand.
#
# It deliberately stops short of loading/restarting anything on the Pi
# itself — that stays a deliberate, manual step (see rewrite-plan.md §11's
# "no automated Pi deployment" stance), printed at the end as a copy-paste
# next step.
#
# Usage:
#   git tag -a v2.0.0 -m "Fermenter v2.0.0"
#   ./scripts/build_and_ship_image.sh pi@raspberrypi.local
#   PI_HOST=pi@raspberrypi.local ./scripts/build_and_ship_image.sh
#
# Optional environment overrides:
#   IMAGE_NAME    (default: fermenter)
#   IMAGE_TAG     (default: the annotated SemVer Git tag at HEAD; use only for
#                  non-release/test builds, e.g. IMAGE_TAG=arm64)
#   PLATFORM      (default: linux/arm64)
#   BUILDER_NAME  (default: fermenter-builder)
#   REMOTE_PATH   (default: ~/)
#   OUTPUT_DIR    (default: dist/, gitignored)
#
# One-time host setup (per slice-7-deployment tasks.md task 2.2), if not
# already done on this machine:
#   docker run --privileged --rm tonistiigi/binfmt --install arm64

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

IMAGE_NAME="${IMAGE_NAME:-fermenter}"
PLATFORM="${PLATFORM:-linux/arm64}"
BUILDER_NAME="${BUILDER_NAME:-fermenter-builder}"
REMOTE_PATH="${REMOTE_PATH:-~/}"
OUTPUT_DIR="${OUTPUT_DIR:-dist}"

PI_HOST="${1:-${PI_HOST:-}}"
if [[ -z "$PI_HOST" ]]; then
  echo "Usage: $0 <user@pi-host>  (or set PI_HOST env var)" >&2
  exit 1
fi

if [[ -z "${IMAGE_TAG:-}" ]]; then
  IMAGE_TAG="$(git -C "$REPO_ROOT" describe --exact-match --tags 2>/dev/null || true)"
  if [[ -z "$IMAGE_TAG" ]]; then
    echo "HEAD must have an annotated SemVer Git tag (for example, v2.0.0)." >&2
    echo "Create and check out the release tag, or set IMAGE_TAG for a non-release/test build." >&2
    exit 1
  fi

  if [[ "$(git -C "$REPO_ROOT" for-each-ref --format='%(objecttype)' "refs/tags/$IMAGE_TAG")" != "tag" ]]; then
    echo "Release tag '$IMAGE_TAG' must be annotated." >&2
    exit 1
  fi

  if [[ ! "$IMAGE_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
    echo "Release tag '$IMAGE_TAG' must use SemVer (for example, v2.0.0)." >&2
    exit 1
  fi
fi

IMAGE_REF="${IMAGE_NAME}:${IMAGE_TAG}"
ARCHIVE_NAME="${IMAGE_NAME}-${IMAGE_TAG}.tar.gz"

echo "==> Ensuring buildx builder '${BUILDER_NAME}' exists (${PLATFORM} support)"
if ! docker buildx inspect "$BUILDER_NAME" >/dev/null 2>&1; then
  docker buildx create --name "$BUILDER_NAME" --driver docker-container --use
else
  docker buildx use "$BUILDER_NAME"
fi

echo "==> Building ${IMAGE_REF} for ${PLATFORM} (this is slow under QEMU emulation, ~10-15 min)"
docker buildx build \
  --builder "$BUILDER_NAME" \
  --platform "$PLATFORM" \
  --tag "$IMAGE_REF" \
  --load \
  "$REPO_ROOT/fermenter"

mkdir -p "$REPO_ROOT/$OUTPUT_DIR"
ARCHIVE_PATH="$REPO_ROOT/$OUTPUT_DIR/$ARCHIVE_NAME"

echo "==> Saving ${IMAGE_REF} to ${ARCHIVE_PATH}"
docker save "$IMAGE_REF" | gzip -c > "$ARCHIVE_PATH"
ls -lh "$ARCHIVE_PATH"

echo "==> Copying ${ARCHIVE_NAME} to ${PI_HOST}:${REMOTE_PATH}"
scp "$ARCHIVE_PATH" "${PI_HOST}:${REMOTE_PATH}"

cat <<EOF

==> Done. On the Pi, run:

    gunzip -c ${REMOTE_PATH%/}/${ARCHIVE_NAME} | docker load
    cd /path/to/fermenter-temp-controller && FERMENTER_IMAGE_TAG=${IMAGE_TAG} docker compose up -d

(docker compose up -d picks up the freshly loaded ${IMAGE_REF} image
without rebuilding, since compose.yaml's fermenter service has both
'image:' and 'build:' set — see compose.yaml's comments.)
EOF
