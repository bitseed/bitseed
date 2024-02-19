
import { Generator } from './types'
import { APIInterface, DeployOptions } from './interfaces'
import { RoochBitSeedApiInterface } from './api'
import { JsonRpcDatasource } from "@sadoprotocol/ordit-sdk";
import { Inscriber, Ordit, ordit } from "@sadoprotocol/ordit-sdk"

export class BitSeed implements APIInterface {
  private primaryWallet: Ordit;
  private fundingWallet: Ordit;
  private datasource: JsonRpcDatasource;
  private bitSeedApi: RoochBitSeedApiInterface;

  constructor(primaryWallet: Ordit, fundingWallet: Ordit, datasource: JsonRpcDatasource, bitSeedApi: RoochBitSeedApiInterface) {
    this.primaryWallet = primaryWallet;
    this.fundingWallet = fundingWallet;
    this.datasource = datasource;
    this.bitSeedApi = bitSeedApi;
  }

  name(): string {
    return "bitseed"
  }
  
  async deploy(tick: string, max: number, generator: Generator, opts?: DeployOptions | undefined): Promise<string> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error("not selected address")
    }

    let generatorURI = '';

    if (typeof generator === 'string') {
      generatorURI = `/content/${generator}`;
    } else if (generator instanceof Uint8Array) {
      const inscriptionID = await this.inscribeWASM(generator, opts);
      generatorURI = `/content/${inscriptionID}`;
    } else {
      throw new Error("Invalid generator type");
    }

    const meta = {
      tick,
      max: max || null,
      generator: generatorURI,
      repeat: opts?.repeat || 0,
      has_user_input: opts?.has_user_input || false,
      deploy_args: opts?.deploy_args || []
    };

    console.log("deploy meta:", meta)

    const transaction = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: this.primaryWallet.selectedAddress,
      mediaContent: JSON.stringify(meta),
      mediaType: "application/json",
      feeRate: 1,
      meta: { // Flexible object: Record<string, any>
        title: "Example title",
        desc: "Lorem ipsum",
        slug: "cool-digital-artifact",
        creator: {
          name: "Your Name",
          email: "artist@example.org",
          address: this.primaryWallet.selectedAddress
        }
      },
      postage: 600 // base value of the inscription in sats
    })

    // generate deposit address and fee for inscription
    const revealed = await transaction.generateCommit();

    // deposit revealFee to address
    console.log("revealed:", revealed) 
    await this.depositRevealFee(revealed)

    // confirm if deposit address has been funded
    if (await transaction.isReady({skipStrictSatsCheck: false})) {
      // build transaction
      await transaction.build();

      // sign transaction
      const signedTxHex = this.primaryWallet.signPsbt(transaction.toHex(), { isRevealTx: true });

      // Broadcast transaction
      const txId = await this.datasource.relay({ hex: signedTxHex });
      console.log("txId:", txId);

      return txId
    } else {
      throw new Error("transaction not ready");
    }
  }

  private async inscribeWASM(wasmBytes: Uint8Array, opts?: DeployOptions): Promise<string> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error("not selected address")
    }

    const base64Wasm = Buffer.from(wasmBytes).toString('base64');

    const wasmInscription = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: this.primaryWallet.selectedAddress,
      mediaContent: base64Wasm,
      mediaType: "application/wasm",
      feeRate: 1,
      meta: { // Flexible object: Record<string, any>
        title: "Example title",
        desc: "Lorem ipsum",
        slug: "cool-digital-artifact",
        creator: {
          name: "Your Name",
          email: "artist@example.org",
          address: this.primaryWallet.selectedAddress
        }
      },
      postage: 600 // base value of the inscription in sats
    });

    const revealed = await wasmInscription.generateCommit();
    console.log(revealed) // deposit revealFee to address

    if (await wasmInscription.isReady({skipStrictSatsCheck: true})) {
      await wasmInscription.build();

      const signedTxHex = this.primaryWallet.signPsbt(wasmInscription.toHex(), { isRevealTx: true });

      const wasmTx = await this.datasource.relay({ hex: signedTxHex });
      return wasmTx as any;
    } else {
      throw new Error("WASM Inscription funding is not ready");
    }
  }

  async depositRevealFee(revealed: {
      address: string;
      revealFee: number;
  }) {
    if (!this.fundingWallet.selectedAddress) {
      throw new Error("not selected address")
    }

    const psbt = await ordit.transactions.createPsbt({
      pubKey: this.fundingWallet.publicKey,
      address: this.fundingWallet.selectedAddress,
      outputs: [{
          address: revealed.address,
          value: revealed.revealFee
      }],
      network: this.fundingWallet.network,
      satsPerByte: 1,
    })

    const signedTxHex = await this.fundingWallet.signPsbt(psbt.hex)
    const txId = await this.datasource.relay({ hex: signedTxHex })

    console.log({ txId })
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
