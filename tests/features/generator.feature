Feature: Bitseed CLI integration tests
  @serial
  Scenario: basic
    # prepare
    Given bitcoind and Ord servers
    Then cmd ord: "wallet create"
    Then cmd ord: "wallet receive"

    # mint utxos
    Then cmd bitcoin-cli: "generatetoaddress 101 {{$.wallet[-1].address}}"
    Then sleep: "5" # wait ord sync and index
    Then cmd ord: "wallet balance"
    Then assert: "{{$.wallet[-1].total}} == 5000000000"

    # generator
    Then cmd bitseed: "generator --fee-rate 1 --name random --generator ./generator/cpp/generator.wasm"
    Then assert: "'{{$.generator[-1]}}' not_contains error"

    # mine a block
    Then cmd ord: "wallet receive"
    Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
    Then sleep: "5"

    # deploy
    Then cmd bitseed: "deploy --fee-rate 1 --generator {{$.generator[-1].inscription.Id}} --tick bits --amount 210000000000 --deploy-args {"height":{"type":"range","data":{"min":1,"max":1000}}}"
    Then assert: "'{{$.deploy[-1]}}' not_contains error"

    # mine a block
    Then cmd ord: "wallet receive"
    Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
    Then sleep: "5"

    # mint 
    Then cmd bitseed: "mint --fee-rate 1 --deploy-inscription-id {{$.deploy[-1].inscription.Id}} --user-input hello_bitseed" 
    Then assert: "'{{$.mint[-1]}}' not_contains error"

    # mine a block
    #Then cmd ord: "wallet receive"
    #Then cmd bitcoin-cli: "generatetoaddress 1 {{$.wallet[-1].address}}"
    #Then sleep: "5"

    # split 
    #Then cmd bitseed: "split --fee-rate 1 --asset-inscription-id {{$.mint[-1].inscription.Id}} --amount 100" 
    #Then assert: "'{{$.split[-1]}}' not_contains error"

    # end
    Then release bitcoind and Ord servers
