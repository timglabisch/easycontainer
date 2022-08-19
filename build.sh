SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

echo "Building Rust Images"
echo "Building Rust AMD64 Image"
CONTAINER_AMD64=$(docker buildx build --platform linux/amd64 $SCRIPT_DIR -q)
echo "Building Rust ARM64 Image"
CONTAINER_ARM64=$(docker buildx build --platform linux/arm64 $SCRIPT_DIR -q)

echo "Building Binaries"
echo "Rust Build Binary (AMD64)"
docker run --rm \
  --entrypoint=cargo \
  --workdir=/build/$2 \
  --platform linux/amd64 \
  -e CARGO_HOME="/cargo" \
  --network host \
  -v "`pwd`/$1:/build" \
  -v "/tmp/.cargo_amd64_cache:/cargo" \
  "$CONTAINER_AMD64" \
  build --release --target x86_64-unknown-linux-musl

echo "Rust Build Binary (ARM64)"
docker run --rm \
  --entrypoint=cargo \
  --workdir=/build/$2 \
  --platform linux/arm64/v8 \
  -e CARGO_HOME="/cargo" \
  --network host \
  -v "`pwd`/$1:/build" \
  -v "/tmp/.cargo_arm64_cache:/cargo" \
  "$CONTAINER_ARM64" \
  build --release --target aarch64-unknown-linux-musl

echo "Moving Build Artifacts"
BINARY_DIR="`pwd`/$2/.tmp_eb_build"
rm -rf "`pwd`/$2/.tmp_eb_build/"
mkdir -p "$BINARY_DIR"
cp -r ./target "$BINARY_DIR"

cd "`pwd`/$2";
echo "Building Containers"
echo "Rust Build Container (AMD64)"

echo "Build AMD64"
docker buildx build \
  --build-arg CARGO_RELEASE=.tmp_eb_build/target/x86_64-unknown-linux-musl/release \
  --platform linux/amd64 \
  .

echo "Build ARM64"
docker buildx build \
  --build-arg CARGO_RELEASE=.tmp_eb_build/target/aarch64-unknown-linux-musl/release \
  --platform linux/arm64/v8 \
  .