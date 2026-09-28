set -eu
# Populate a fresh runner cache before the offline package checks.
cargo fetch
package_build=$(mktemp -d /results/bootstrap-package.XXXXXX)
CARGO_TARGET_DIR="$package_build" cargo package --allow-dirty --no-verify
mkdir -p /cache/recording-crate
tar -xzf "$package_build/package/reacon-sdk-$REACON_SDK_PACKAGE_VERSION.crate" -C /cache/recording-crate
cargo fetch --manifest-path /results/consumer/Cargo.toml
sh /suite/rust-package.sh
