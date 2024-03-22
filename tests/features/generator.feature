Feature: Bitseed CLI integration tests
  @serial
  Scenario: demo
    Given bitcoind and Ord servers
    Then cmd ord: "wallet create"
    Then cmd ord: "wallet receive"
    Then cmd bitcoin-cli: "generatetoaddress 101 {{$.wallet[-1].address}}"
    Then sleep: "5" # wait ord sync and index
    Then cmd ord: "wallet balance"
    Then assert: "{{$.wallet[-1].total}} == 5000000000"
    Then cmd bitseed: "generator --fee-rate 1 --name random --generator ../../generator/generator.wasm"
    Then assert: "'{{$.generator[-1]}}' not_contains error"
    Then release bitcoind and Ord servers
