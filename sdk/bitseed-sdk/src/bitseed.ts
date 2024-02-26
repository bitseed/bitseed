import { JsonRpcDatasource } from '@sadoprotocol/ordit-sdk'
import { Inscriber, Ordit, ordit } from '@sadoprotocol/ordit-sdk'

import { BITSEED_PROTOAL_NAME } from './constants'
import { InscriptionID, Generator, Tick, SFTRecord } from './types'
import { inscriptionIDToString, toB64 } from './utils'
import { IGeneratorLoader } from './generator'
import { APIInterface, DeployOptions, InscribeOptions } from './interfaces'

export class BitSeed implements APIInterface {
  private primaryWallet: Ordit
  private fundingWallet: Ordit
  private datasource: JsonRpcDatasource
  private generatorLoader: IGeneratorLoader

  constructor(
    primaryWallet: Ordit,
    fundingWallet: Ordit,
    datasource: JsonRpcDatasource,
    generatorLoader: IGeneratorLoader,
  ) {
    this.primaryWallet = primaryWallet
    this.fundingWallet = fundingWallet
    this.datasource = datasource
    this.generatorLoader = generatorLoader
  }

  protected async inscribe(sft: SFTRecord, opts?: InscribeOptions): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    let meta = {
      op: sft.op,
      tick: sft.tick,
      amount: sft.amount,
      attributes: sft.attributes
    }

    let contentType: string | undefined = undefined
    let body: string | undefined = undefined

    if (sft.content) {
      contentType = sft.content.content_type
      body = toB64(sft.content.body)
    }

    const inscriber = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: body,
      mediaType: contentType,
      feeRate: opts?.fee_rate || 1,
      meta: meta,
      postage: opts?.postage || 1000, // base value of the inscription in sats
    })

    inscriber.withMetaProtocol(BITSEED_PROTOAL_NAME)

    const revealed = await inscriber.generateCommit()
    console.log("revealed:", revealed)

    // deposit revealFee to address
    await this.depositRevealFee(revealed, opts)

    let ready = false;

    try {
      await inscriber.fetchAndSelectSuitableUnspent({ skipStrictSatsCheck: true })
      ready = true
    } catch (error) {
      console.log("inscribe error:", error)
      ready = false
    }

    if (ready || await inscriber.isReady({ skipStrictSatsCheck: true })) {
      await inscriber.build()

      const signedTxHex = this.primaryWallet.signPsbt(inscriber.toHex(), { isRevealTx: true })

      const inscribeTx = await this.datasource.relay({ hex: signedTxHex })

      return {
        txid: inscribeTx,
        index: 0,
      }
    } else {
      throw new Error('inscriber is not ready')
    }
  }

  protected async depositRevealFee(
    revealed: {
      address: string
      revealFee: number
    },
    opts?: InscribeOptions,
  ) {
    if (!this.fundingWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    const psbt = await ordit.transactions.createPsbt({
      pubKey: this.fundingWallet.publicKey,
      address: this.fundingWallet.selectedAddress,
      outputs: [
        {
          address: revealed.address,
          value: revealed.revealFee,
        },
      ],
      network: this.fundingWallet.network,
      satsPerByte: opts?.commit_fee_rate || opts?.fee_rate || 1,
    })

    const signedTxHex = await this.fundingWallet.signPsbt(psbt.hex)
    const txId = await this.datasource.relay({ hex: signedTxHex })

    console.log('depositRevealFee txId:', txId)
  }

  public async generator(wasmBytes: Uint8Array, opts?: InscribeOptions): Promise<InscriptionID> {
    const sft: SFTRecord = {
      op: "mint",
      tick: "generator",
      amount: 1,
      content: {
        content_type: 'application/wasm',
        body: wasmBytes
      }
    }

    return this.inscribe(sft, opts)
  }

  public async deploy(
    tick: string,
    max: number,
    generator: Generator,
    opts?: DeployOptions | undefined,
  ): Promise<InscriptionID> {
    const sft: SFTRecord = {
      op: "deploy",
      tick: tick,
      amount: max,
      attributes: {
        repeat: opts?.repeat || 0,
        generator: `/inscription/${generator}`,
        has_user_input: opts?.has_user_input || false,
        deploy_args: opts?.deploy_args
      }
    }

    return this.inscribe(sft, opts)
  }

  public async mint(
    tickInscriptionId: InscriptionID,
    userInput: string,
    opts?: InscribeOptions,
  ): Promise<InscriptionID> {
    if (!opts?.satpoint) {
      throw new Error('mint must set satpoint')
    }

    let tick = await this.getTickByInscriptionId(tickInscriptionId)
    const generator = await this.generatorLoader.load(tick.generator)
    const sft = await generator.inscribeGenerate(tick.deploy_args, opts?.satpoint, userInput)
    console.log('SFT record:', sft)

    return this.inscribe(sft, opts)
  }

  private async getTickByInscriptionId(inscription_id: InscriptionID): Promise<Tick> {
    const tickInscription = await this.datasource.getInscription({
      id: inscriptionIDToString(inscription_id),
      decodeMetadata: false,
    })

    const tick = JSON.parse(tickInscription.mediaContent) as Tick
    return tick
  }

  public async merge(a: InscriptionID, b: InscriptionID): Promise<InscriptionID> {
    throw new Error('Method not implemented.')
  }

  public async split(a: InscriptionID): Promise<[InscriptionID, InscriptionID]> {
    throw new Error('Method not implemented.')
  }
}
