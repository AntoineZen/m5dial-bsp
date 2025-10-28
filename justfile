

default:
    just -l

# Prepare documentation for publishing GitHub pages
publish-doc:
    cargo clean
    cargo doc --no-deps
    git checkout gh-pages
    cp -r target/xtensa-esp32s3-none-elf/doc/* .
    git add -u

# Run screen_counter example.
run-counter:
    cargo run --example screen_counter


# Run touch screen example.
run-touch:
    cargo run --example touch
