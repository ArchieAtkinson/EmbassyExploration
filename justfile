# Flash App
[working-directory: 'app']
run:
    cargo run

# Host Test
[working-directory: 'common-lib']
htest:
    cargo test

# Target Test
[working-directory: 'hw-lib']
ttest:
    cargo test
