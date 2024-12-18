#!/bin/sh

run() {
  echo "Building the project and running the executable..."
  cargo build --release
  sudo target/release/rstracer
}

test() {
  echo "Running tests..."
  cargo test --lib
}

lint() {
  echo "Running linter..."
  git init
  git add .
  pre-commit run --all-files
}

coverage() {
  echo "Generating code coverage report..."
  rustup toolchain add nightly --component llvm-tools-preview
  rustup override set nightly
  LLVM_PROFILE_FILE='.coverage/grcov-%p-%m.profraw' RUSTFLAGS='-Cinstrument-coverage' cargo test
  grcov $(find . -name "grcov-*.profraw" -print) \
    --branch \
    --ignore-not-existing \
    --binary-path ./target/debug/ \
    -s . \
    -t lcov \
    --ignore "/*" \
    -o .coverage/lcov.info
  echo "Coverage report generated at .coverage/lcov.info"
}

if [ $# -eq 0 ]; then
  echo "Usage: $0 {run|test|coverage}"
  exit 1
fi

case "$1" in
  run)
    run
    ;;
  test)
    test
    ;;
  lint)
    lint
    ;;
  coverage)
    coverage
    ;;
  *)
    echo "Invalid argument: $1"
    echo "Usage: $0 {run|test|lint|coverage}"
    exit 1
    ;;
esac
