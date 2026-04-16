# Generic Rust workspace tasks.
# Ply-specific commands for building and running the editor.

default: check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

tokei-check:
    ./tokei_check.sh

tokei-check-editor:
    ./tokei_check.sh 800 500 apps/taled-editor/src

check:
    cargo check --workspace --all-targets --all-features

test:
    cargo test --workspace --all-features

nextest:
    cargo nextest run --workspace

fix:
    cargo clippy --workspace --fix --allow-dirty --allow-staged --all-features

deny:
    cargo deny check

audit:
    cargo audit

clean:
    cargo clean

run:
    cargo run

ply-serve:
    plyx serve

ssh-preview:
    ./scripts/ssh-preview.sh

ply-build:
    plyx build

android-build:
    plyx apk -p taled-editor --release
    python3 ./scripts/patch_android_icons.py
    cd target/dx/taled-editor/release/android/app && ./gradlew :app:assembleDebug

ply-bundle:
    plyx bundle

ui-compare:
    ./scripts/ui_compare.sh

perf-check sample="Theater":
    ./scripts/perf_check.sh '{{sample}}'

perf-check-theater:
    ./scripts/perf_check.sh 'Theater'

perf-check-frontier:
    ./scripts/perf_check.sh 'Existential Frontier'
