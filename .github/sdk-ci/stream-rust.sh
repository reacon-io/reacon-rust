set -eu
test ! -e /work
mkdir -p /cache/stream-crate /results/consumer
archive="/artifacts/reacon-sdk-$REACON_SDK_PACKAGE_VERSION.crate"
tar -xzf "$archive" -C /cache/stream-crate
cp -r /sdk/conformance/stream-rust/. /results/consumer/
sed -i "s|path = \"/work\"|path = \"/cache/stream-crate/reacon-sdk-$REACON_SDK_PACKAGE_VERSION\"|" /results/consumer/Cargo.toml
host=$(rustc -vV | sed -n 's/^host: //p')
cargo metadata --offline --filter-platform "$host" --manifest-path /results/consumer/Cargo.toml --format-version 1 > /results/cargo-metadata.json
cargo run --offline --manifest-path /results/consumer/Cargo.toml
