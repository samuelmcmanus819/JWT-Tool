# JWT Tool
## Introduction
The JWT tool is a tool to provision and verify JWTs implemented as a SCRT smart contract.

The primary use-case of this tool is if you want to set up an off-chain wallet address-based authorization scheme. This contract prevents the user from needing to do complex things to provision a JWT, like signing and validating off-chain transactions with a user's wallet. 

With the JWT tool, the user requests a JWT from the contract. Then when they include the JWT in their future requests to the web server, the web server validates the JWT by running a validation query. Alternatively, if you'd like to save some time, then you can get the public key from this contract and validate JWTs on the web server on your own!

## Usage
This documentation assumes that you have a properly-configured SCRT development environment where you can post to the SCRT mainnet. You should have your SecretCLI configured to use mainnet as follows:
```
secretcli config node https://rpc-secret.keplr.app
secretcli config chain-id secret-4
```
### Deploy the Contract
1. Build the project from the root directory.
```
cargo test
```
2. Run the optimizer to optimize the build
```
docker run --rm -v "$(pwd)":/contract   --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target   --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry   enigmampc/secret-contract-optimizer 
```
3. Store your code on the SCRT chain
```
secretcli tx compute store contract.wasm.gz --gas auto --gas-adjustment 1.3 --from <your-wallet-name>
```
4. Verify success by listing the code and verifying that the last code id was created by your wallet address
```
secretcli query compute list-code
```
5. Instantiate the contract
```
secretcli tx compute instantiate <code-id> '{"hours_until_token_expiration": <desired_token_ttl_in_hours>}' --from <your-wallet-name> -b block --label jwt-tool -y
```
6. Get the contract address
```
secretcli query compute list-contract-by-code <code-id>
```
### Get a JWT
1. Provision the JWT
```
secretcli tx wasm execute <contract-address> '{"provision": { } }'  --from <your-wallet-name> -b block
```
2. Get the JWT by grabbing the TX hash and querying it **note: this is not required if using secretjs with Keplr**
```
secretcli q compute tx $TXHASH
```
3. Pull the value of the `jwt` from the response.
### Contractually Validate the JWT
Now that you have your JWT, you can query the contract to check its validity.
```
secretcli q compute query <contract-address> '{ "validate_jwt": { "jwt": "<your-jwt>" } }'
```
### Validate a JWT on Your Own
1. Grab the public key of the contract
```
secretcli q compute query secret16xsk80vxhah3p0smj3cwl9kxttrcwsrll40rxr '{ "get_pub_key": { } }'
```
2. Validate the signature by decrypting the signature with the public key, computing a SHA-256 hash of the decoded header + payload, and comparing decrypted signature to the hash.
