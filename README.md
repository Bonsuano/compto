# Environment

Ubuntu 22.04  

# Build

`cargo build-sbf`  

# Local Environment

## Dependencies

Install rust and the solana CLI (see docs: https://solana.com/tr/developers/guides/getstarted/setup-local-development)  

```bash
sudo apt install pre-commit
pre-commit install
```

## Test Dependencies

python ^3.11.6  
navigate to `test/compt-test-client` and run `npm install`  

# Testing

`pip install -r test/requirements.txt`  

## unit tests

run `cargo test-sbf`  

## component tests

ensure that the `comptoken.so` file is in one of these directories  
 - `./tests/fixtures`  
 - the current working directory  
 - a directory defined in the `BPF_OUT_DIR` or `SBF_OUT_DIR` environment variables  

we recommend setting `SBF_OUT_DIR` to `<path-to-compto>/target/deploy/`  

use the integration test deployment script to compile the program  
<!--compile the program with `cargo build-sbf --features testmode`  -->

run with `node test/compto-test-client/<test>`  

component tests  
 - test_mint
 - initialize_comptoken_program
 - test_getValidBlockhashes
 - test_createUserDataAccount

## integration tests

run the test deployment script: `python3 test/full_deploy_test.py`  

# Debugging

View logs emmitted from failures in the solana program with `solana logs --commitment max`  
