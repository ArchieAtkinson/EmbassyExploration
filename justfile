TARGET := "thumbv7em-none-eabihf"
TARGET_DIR := "target/mcu"
HOST_DIR := "target/host"

# Flash App
[working-directory: "app"]
@flash *ARGS:
    cargo run --target-dir={{TARGET_DIR}} {{ARGS}}

@check *ARGS:
    echo "\n ------------- App Output ------------- \n"
    cargo check --target {{TARGET}} --target-dir={{TARGET_DIR}} {{ARGS}}


# Host Test
@htest *ARGS: hcheck
    RUST_LOG=debug cargo test -p common-lib --target-dir={{HOST_DIR}} {{ARGS}}


@hcheck:
    cargo check -p common-lib --target {{TARGET}} --target-dir={{TARGET_DIR}}

# Host Coverage
@hcov:
    cargo tarpaulin -p common-lib --lib 

# Target Test
@ttest:
    cargo test -p hw-lib
