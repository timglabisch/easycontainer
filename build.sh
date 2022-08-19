SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
echo "Building AMD64 Image"
CONTAINER_AMD64=$(docker buildx build --platform linux/amd64 $SCRIPT_DIR -q)
echo "Building ARM64 Image"
CONTAINER_ARM64=$(docker buildx build --platform linux/arm64 $SCRIPT_DIR -q)

echo "Cargo AMD64"
docker run --rm \
  --entrypoint=cargo \
  --workdir=/build/$2 \
  --platform linux/amd64 \
  --network host \
  -v "`pwd`/$1:/build" \
  "$CONTAINER_AMD64" \
  build --release --target x86_64-unknown-linux-musl

echo "Cargo ARM64"
docker run --rm \
  --entrypoint=cargo \
  --workdir=/build/$2 \
  --platform linux/arm64/v8 \
  --network host \
  -v "`pwd`/$1:/build" \
  "$CONTAINER_ARM64" \
  build --release --target aarch64-unknown-linux-musl
