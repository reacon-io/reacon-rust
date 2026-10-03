set -eu
test ! -e /work
mkdir -p /results/consumer
archive="/artifacts/reacon-sdk-$REACON_SDK_PACKAGE_VERSION.crate"
digest=$(sha256sum "$archive" | cut -d ' ' -f 1)
package_root=/cache/recording-crate/$digest
# Restore the retained archive at the same content-addressed path. Its recorded
# mtimes preserve Cargo's identity, while replacement removes modified/extra files
# left by the response consumer. The source checkout remains unavailable.
rm -rf "$package_root"
mkdir -p "$package_root"
tar -xzf "$archive" -C "$package_root"
cp -r /sdk/conformance/stream-rust/. /results/consumer/
sed -i "s|path = \"/work\"|path = \"$package_root/reacon-sdk-$REACON_SDK_PACKAGE_VERSION\"|" /results/consumer/Cargo.toml
host=$(rustc -vV | sed -n 's/^host: //p')
cargo metadata --offline --filter-platform "$host" --manifest-path /results/consumer/Cargo.toml --format-version 1 > /results/cargo-metadata.json
cargo run --offline --manifest-path /results/consumer/Cargo.toml
