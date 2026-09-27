set -eu
# Populate a fresh runner cache before the offline package checks.
cargo fetch
cargo package --allow-dirty --no-verify
mkdir -p /cache/recording-crate
tar -xzf "/cache/target/package/reacon-sdk-$REACON_SDK_PACKAGE_VERSION.crate" -C /cache/recording-crate
cargo fetch --manifest-path /results/consumer/Cargo.toml
sh /suite/rust-package.sh
