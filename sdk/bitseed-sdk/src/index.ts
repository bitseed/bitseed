
import { Generator } from './types'
import { APIInterface, DeployOptions } from './interfaces'
import { RoochBitSeedApiInterface } from './api'
import { JsonRpcDatasource } from "@sadoprotocol/ordit-sdk";
import { Inscriber, Ordit } from "@sadoprotocol/ordit-sdk"

export class BitSeed implements APIInterface {
  private network: string;
  private wallet: Ordit;
  private datasource: JsonRpcDatasource;
  private bitSeedApi: RoochBitSeedApiInterface;

  constructor(wallet: Ordit, datasource: JsonRpcDatasource, bitSeedApi: RoochBitSeedApiInterface) {
    this.network = "testnet"
    this.wallet = wallet;
    this.datasource = datasource;
    this.bitSeedApi = bitSeedApi;
  }

  async deploy(tick: string, max: number, generator: Generator, opts?: DeployOptions | undefined): Promise<string> {
    if (!this.wallet.selectedAddress) {
      throw new Error("not selected address")
    }

    // new inscription tx
    const transaction = new Inscriber({
      network: this.network as any,
      address: this.wallet.selectedAddress,
      publicKey: this.wallet.publicKey,
      changeAddress: this.wallet.selectedAddress,
      destinationAddress: this.wallet.selectedAddress,
      mediaContent: 'Hello World',
      mediaType: "text/plain",
      feeRate: 3,
      meta: { // Flexible object: Record<string, any>
        title: "Example title",
        desc: "Lorem ipsum",
        slug: "cool-digital-artifact",
        creator: {
          name: "Your Name",
          email: "artist@example.org",
          address: this.wallet.selectedAddress
        }
      },
      postage: 1500 // base value of the inscription in sats
    })

    // generate deposit address and fee for inscription
    const revealed = await transaction.generateCommit();
    console.log(revealed) // deposit revealFee to address

    // confirm if deposit address has been funded
    const ready = await transaction.isReady();

    if (ready || await transaction.isReady()) {
      // build transaction
      await transaction.build();

      // sign transaction
      const signedTxHex = this.wallet.signPsbt(transaction.toHex(), { isRevealTx: true });

      // Broadcast transaction
      const tx = await this.datasource.relay({ hex: signedTxHex });
      console.log(tx);
    }

    return ""
  }

  mint(tick: string, amt: number, attributes?: Map<string, string> | undefined): Promise<string> {
    throw new Error('Method not implemented.')
  }

  merge(a: string, b: string): Promise<string> {
    throw new Error('Method not implemented.')
  }

  split(a: string): Promise<[string, string]> {
    throw new Error('Method not implemented.')
  }

}
