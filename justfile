# Flash App
[working-directory: 'app']
@run:
    cargo run

[working-directory: 'app']
@check_app:
    echo "\n ------------- App Output ------------- \n"
    cargo check



# Host Test
[working-directory: 'common-lib']
@htest *ARGS: check_app
    echo "\n ------------- Test Output ------------- \n"
    RUST_LOG=debug cargo test {{ARGS}} 

# Host Coverage
[working-directory: 'common-lib']
@hcov:
    cargo tarpaulin --lib 

# Target Test
[working-directory: 'hw-lib']
@ttest: check_app
    cargo test
