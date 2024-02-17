import React, { useEffect, useState } from "react"
import { JsonRpcDatasource } from "@sadoprotocol/ordit-sdk";
import { Ordit } from "@sadoprotocol/ordit-sdk"
import { BitSeed, BitSeedApiMock } from '../../src'

const MNEMONIC = "<mnemonic>"
const network = "testnet"
const datasource = new JsonRpcDatasource({ network })
const bitseedApiMock = new BitSeedApiMock();

export default function DeployButton() {
  const [bitseed, setBitseed] = useState<BitSeed|undefined>(undefined);

  useEffect(()=>{
    const wallet = new Ordit({
      bip39: MNEMONIC,
      network
    });
  
    wallet.setDefaultAddress('taproot')
  
    const bitseed = new BitSeed(wallet, datasource, bitseedApiMock);
    setBitseed(bitseed)
  }, [])

  return (
    <div>
      Deploy: {bitseed?.name()}
    </div>
  );
}
