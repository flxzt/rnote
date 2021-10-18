#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export OUTPUT="$3"
export PROFILE="$4"
export APP_BIN="$5"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$MESON_BUILD_ROOT"/cargo-home

echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"
echo "CARGO_HOME: $CARGO_HOME"

if [[ $PROFILE = "devel" ]]
then
    echo -e "\n--- DEVEL PROFILE ---\n"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/"$APP_BIN" "$OUTPUT"
else
    echo -e "\n--- RELEASE PROFILE ---\n"

    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/"$APP_BIN" "$OUTPUT"
fi