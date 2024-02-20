# Bitcoin local development environment

## Prepare a local Bitcoin development environment

1. Follow the instructions in the https://github.com/rooch-network/rooch/tree/main/scripts/bitcoin
2. Compile and install bitseed via run `cargo install --path .` in the bitseed directory.
3. Export a command alias to run bitseed command.
```bash
alias bitseed="bitseed --regtest --rpc-url http://127.0.0.1:18443 --bitcoin-rpc-user roochuser --bitcoin-rpc-pass roochpass"
```

## Usage

Start a terminal and run the following commands:

```bash
ord server
``` 

to start the ord server

Start another terminal and run the following commands:

1. Run `ord wallet create` to create a new ord wallet
2. Run `ord wallet receive` to get a new address to receive funds
3. Run `bitcoin-cli generatetoaddress 101 <address>` to generate 101 blocks to the address
4. Run `ord wallet balance` to check the balance of the wallet
5. Run `echo "Hello rooch">/tmp/hello.txt` to create a file
6. Run `bitseed generator --fee-rate 1 --name test --generator generator/generator.wasm` to inscribe the file to the blockchain
7. Run `bitcoin-cli generatetoaddress 1 <address>` to mine an inscription
8. Run `bitseed deploy --fee-rate 1 --generator $the_inscription_from_pre_step --tick bits --amount 210000000000 --deploy-args 1000 --deploy-args 100000`
9. Run `bitcoin-cli generatetoaddress 1 <address>` to mine  an inscription
10. Run `bitseed mint --deploy-inscription-id $the_inscription_from_pre_step --fee-rate 1`
11. Run `bitcoin-cli generatetoaddress 1 <address>` to mine  an inscription
12. Run `ord wallet inscriptions` to get the inscriptions

### References
* [Bitcoin Core](https://bitcoincore.org/en/doc/25.0.0/)
* [ord testing](https://docs.ordinals.com/guides/testing.html): for testing ord inscriptions