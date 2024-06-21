# Environment

Ubuntu 22.04

# Build

`cargo build-sbf`

# Local Environment

## Dependencies

`apt install python3`  
Install rust and the solana CLI (see docs: https://solana.com/tr/developers/guides/getstarted/setup-local-development)  

## Test Dependencies

navigate to `test/compt-test-client` and run `npm install`  

# Testing

then, run the test deployment script: `python3 test/full_deploy_test.py`  


# Debugging

View logs emmitted from failures in the solana program with `solana logs --commitment max`  
