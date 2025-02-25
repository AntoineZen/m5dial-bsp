

default:
    just -l

# Prepare doucmenation for publishing GitHub pages
publish-doc:
    cargo doc --no-deps
    git checkout gh-pages
    cp -r target/xtensa-esp32s3-none-elf/doc/* .
    git add -u

# Run screen_counter example.
run:
    cargo run --release --example screen_counter
