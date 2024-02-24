import { JsonRpcDatasource } from '@sadoprotocol/ordit-sdk'
import { Inscriber, Ordit, ordit } from '@sadoprotocol/ordit-sdk'

import { InscriptionID, Generator, Tick, SFTRecord } from './types'
import { inscriptionIDToString, toB64 } from './utils'
import { APIInterface, DeployOptions, InscribeOptions } from './interfaces'
import { IGeneratorLoader } from './generator'

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

  public async generator(wasmBytes: Uint8Array, opts?: InscribeOptions): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    const base64Wasm = toB64(wasmBytes)

    const wasmInscription = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: base64Wasm,
      mediaType: 'application/wasm',
      feeRate: opts?.fee_rate || 1,
      meta: opts?.meta || {},
      postage: opts?.postage || 600, // base value of the inscription in sats
    })

    const revealed = await wasmInscription.generateCommit()

    // deposit revealFee to address
    console.log('revealed:', revealed)
    await this.depositRevealFee(revealed, opts)

    if (await wasmInscription.isReady({ skipStrictSatsCheck: false })) {
      await wasmInscription.build()

      const signedTxHex = this.primaryWallet.signPsbt(wasmInscription.toHex(), { isRevealTx: true })

      const wasmTx = await this.datasource.relay({ hex: signedTxHex })

      return {
        txid: wasmTx,
        index: 0,
      }
    } else {
      throw new Error('WASM Inscription funding is not ready')
    }
  }

  public async inscribe(sft: SFTRecord, opts?: InscribeOptions): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    let meta = {
      ...sft.attributes
    }

    let content = {
      content_type: "",
      body: new Uint8Array()
    }

    if (sft.content) {
      content = sft.content
    }

    const base64Wasm = toB64(content.body)

    const wasmInscription = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: base64Wasm,
      mediaType: content.content_type,
      feeRate: opts?.fee_rate || 1,
      meta: meta,
      postage: opts?.postage || 600, // base value of the inscription in sats
    })

    const revealed = await wasmInscription.generateCommit()

    // deposit revealFee to address
    console.log('revealed:', revealed)
    await this.depositRevealFee(revealed, opts)

    if (await wasmInscription.isReady({ skipStrictSatsCheck: false })) {
      await wasmInscription.build()

      const signedTxHex = this.primaryWallet.signPsbt(wasmInscription.toHex(), { isRevealTx: true })

      const wasmTx = await this.datasource.relay({ hex: signedTxHex })

      return {
        txid: wasmTx,
        index: 0,
      }
    } else {
      throw new Error('inscriber is not ready')
    }
  }

  public async deploy(
    tick: string,
    max: number,
    generator: Generator,
    opts?: DeployOptions | undefined,
  ): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    let generatorURI = `/content/${inscriptionIDToString(generator)}`

    const meta = {
      tick,
      max: max || null,
      generator: generatorURI,
      repeat: opts?.repeat || 0,
      has_user_input: opts?.has_user_input || false,
      deploy_args: opts?.deploy_args || [],
    }

    console.log('deploy meta:', meta)

    const transaction = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: JSON.stringify(meta),
      mediaType: 'application/json',
      feeRate: opts?.fee_rate || 1,
      meta: opts?.meta || {},
      postage: opts?.postage || 600, // base value of the inscription in sats
    })

    // generate deposit address and fee for inscription
    const revealed = await transaction.generateCommit()

    // deposit revealFee to address
    console.log('revealed:', revealed)
    await this.depositRevealFee(revealed, opts)

    let ready = true
    try {
      await transaction.fetchAndSelectSuitableUnspent({ skipStrictSatsCheck: false })
    } catch (error) {
      console.log('fetchAndSelectSuitableUnspent erorr:', error)
      ready = false
    }

    // confirm if deposit address has been funded
    if (ready || (await transaction.isReady({ skipStrictSatsCheck: false }))) {
      // build transaction
      await transaction.build()

      // sign transaction
      const signedTxHex = this.primaryWallet.signPsbt(transaction.toHex(), { isRevealTx: true })

      // Broadcast transaction
      const txId = await this.datasource.relay({ hex: signedTxHex })
      console.log('txId:', txId)

      return {
        txid: txId,
        index: 0,
      }
    } else {
      throw new Error('transaction not ready')
    }
  }

  private async depositRevealFee(
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

  public async mint(
    tickInscriptionId: InscriptionID,
    userInput: string,
    opts?: InscribeOptions,
  ): Promise<InscriptionID> {
    if (!this.primaryWallet.selectedAddress) {
      throw new Error('not selected address')
    }

    if (!opts?.satpoint) {
      throw new Error('mint must set satpoint')
    }

    let tick = await this.getTickByInscriptionId(tickInscriptionId)
    const generator = await this.generatorLoader.load(tick.generator)
    const meta = await generator.inscribeGenerate(tick.deploy_args, opts?.satpoint, userInput)

    console.log('SFT meta:', meta)

    const transaction = new Inscriber({
      network: this.primaryWallet.network,
      address: this.primaryWallet.selectedAddress,
      publicKey: this.primaryWallet.publicKey,
      changeAddress: this.primaryWallet.selectedAddress,
      destinationAddress: opts?.destination || this.primaryWallet.selectedAddress,
      mediaContent: JSON.stringify(meta),
      mediaType: 'application/json',
      feeRate: opts?.fee_rate || 1,
      meta: opts?.meta || {},
      postage: opts?.postage || 600, // base value of the inscription in sats
    })

    // generate deposit address and fee for inscription
    const revealed = await transaction.generateCommit()

    // deposit revealFee to address
    console.log('revealed:', revealed)
    await this.depositRevealFee(revealed, opts)

    let ready = true
    try {
      await transaction.fetchAndSelectSuitableUnspent({ skipStrictSatsCheck: false })
    } catch (error) {
      console.log('fetchAndSelectSuitableUnspent erorr:', error)
      ready = false
    }

    // confirm if deposit address has been funded
    if (ready || (await transaction.isReady({ skipStrictSatsCheck: false }))) {
      // build transaction
      await transaction.build()

      // sign transaction
      const signedTxHex = this.primaryWallet.signPsbt(transaction.toHex(), { isRevealTx: true })

      // Broadcast transaction
      const txId = await this.datasource.relay({ hex: signedTxHex })
      console.log('txId:', txId)

      return {
        txid: txId,
        index: 0,
      }
    } else {
      throw new Error('transaction not ready')
    }
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
