Feature: Bitseed CLI integration tests
  @serial
  Scenario: demo
    Given bitcoind and Ord servers
    Then cmd ord: "wallet create"
    Then cmd ord: "wallet receive"
    Then cmd bitcoind: "bitcoin-cli generatetoaddress 101 tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h"
    Then cmd ord: "wallet balance"
    Then cmd bitseed: "generator --fee-rate 1 --name random --generator ../../generator/generator.wasm"
    Then release bitcoind and Ord servers
