#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export OUTPUT="$3"
export PROFILE="$4"
export APP_BIN="$5"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$CARGO_TARGET_DIR"/cargo-home

if [[ $4 = "Devel" ]]
then
    echo -e "\nDEVEL PROFILE\n"

    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml && \
        cp "$CARGO_TARGET_DIR"/debug/"$APP_BIN" "$OUTPUT"
else
    echo -e "\nDEFAULT PROFILE\n"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/"$APP_BIN" "$OUTPUT"
fi