import { Decimal } from 'decimal.js';
import { Transaction as BTCTransaction } from "bitcoinjs-lib";
import { GetBalanceOptions, GetInscriptionOptions, GetInscriptionsOptions, GetInscriptionUTXOOptions, GetSpendablesOptions, GetTransactionOptions, GetUnspentsOptions, GetUnspentsResponse, IDatasource, Inscription, RelayOptions, Transaction, UTXO, UTXOLimited } from "@sadoprotocol/ordit-sdk";
import { IUniSatOpenAPI, unisatTypes } from "../../api";
import { decodeScriptPubKey } from '../../utils/bitcoin';
import { toB64 } from '../../utils';
export class UniSatDataSource implements IDatasource {
  private unisatOpenAPI: IUniSatOpenAPI

  constructor(unisatOpenAPI: IUniSatOpenAPI) {
    this.unisatOpenAPI = unisatOpenAPI;
  }

  async getBalance({ address }: GetBalanceOptions): Promise<number> {
    const balance = await this.unisatOpenAPI.getAddressBalance(address);
    const amount: Decimal = new Decimal(balance.amount);
    return amount.toNumber();
  }
  
  async getInscriptionUTXO({ id }: GetInscriptionUTXOOptions): Promise<UTXO> {
    const utxo = await this.unisatOpenAPI.getInscriptionUtxo(id);

    return {
      n: utxo.vout,
      txid: utxo.txid,
      sats: utxo.satoshis,
      scriptPubKey: decodeScriptPubKey(utxo.scriptPk, this.unisatOpenAPI.getNetwork()),
      safeToSpend: utxoSpendable(utxo),
      confirmation: -1,
    }
  }

  async getInscription({ id, decodeMetadata }: GetInscriptionOptions): Promise<Inscription> {
    const utxoDetail = await this.unisatOpenAPI.getInscriptionUtxoDetail(id);
    console.log('utxoDetail:', utxoDetail)

    if (!utxoDetail || utxoDetail.inscriptions.length == 0) {
      throw new Error('inscription nil')
    }

    const inscription = utxoDetail.inscriptions[0]
    const content = await this.unisatOpenAPI.loadContent(inscription.content)
    const base64Content = toB64(new Uint8Array(content))

    let meta = ""
    if (decodeMetadata && utxoDetail && utxoDetail.inscriptions.length >= 2 ) {
      const metaInscription = utxoDetail.inscriptions[1]
      const metaBody = await this.unisatOpenAPI.loadContent(metaInscription.content)
      meta = toB64(new Uint8Array(metaBody))
      console.log("meta:", meta)
    }

    return {
      id: inscription.inscriptionId,
      outpoint: inscription.output,
      owner: inscription.address,
      genesis: inscription.genesisTransaction,
      fee: -1,
      height: inscription.utxoHeight,
      number: inscription.inscriptionNumber,
      sat: utxoDetail.satoshis,
      timestamp: inscription.timestamp,
      mediaType: inscription.contentType,
      mediaSize: inscription.contentLength,
      mediaContent: base64Content,
      value: inscription.outputValue,
    }
  }

  async getInscriptions({ owner, limit }: GetInscriptionsOptions): Promise<Inscription[]> {
    if (!owner) {
      throw new Error('owner is undefine')
    }

    const resp = await this.unisatOpenAPI.getAddressInscriptions(owner, 0, limit || 10)
    return Array.from(resp.list).map((inscription)=>{
      return {
        id: inscription.inscriptionId,
        outpoint: inscription.output,
        owner: inscription.address,
        genesis: inscription.genesisTransaction,
        fee: -1,
        height: inscription.utxoHeight,
        number: inscription.inscriptionNumber,
        sat: -1,
        timestamp: inscription.timestamp,
        mediaType: inscription.contentType,
        mediaSize: inscription.contentLength,
        mediaContent: inscription.contentBody,
        value: inscription.outputValue,
      }
    })
  }

  async getTransaction({ txId }: GetTransactionOptions): Promise<{ tx: Transaction; rawTx?: BTCTransaction | undefined; }> {
    const tx = await this.unisatOpenAPI.getTx(txId)
    return {
      tx,
    }
  }

  async getSpendables({ address }: GetSpendablesOptions): Promise<UTXOLimited[]> {
    const utxos = await this.unisatOpenAPI.getBTCUtxos(address)
    return Array.from(utxos).map((utxo)=>{
      return {
        n: utxo.vout,
        txid: utxo.txid,
        sats: utxo.satoshis,
        scriptPubKey: decodeScriptPubKey(utxo.scriptPk, this.unisatOpenAPI.getNetwork()),
      }
    })
  }

  async getUnspents({ address }: GetUnspentsOptions): Promise<GetUnspentsResponse> {
    const utxos = await this.unisatOpenAPI.getBTCUtxos(address)
    const spendableUTXOs = Array.from(utxos).map((utxo)=>{
      return {
        n: utxo.vout,
        txid: utxo.txid,
        sats: utxo.satoshis,
        scriptPubKey: decodeScriptPubKey(utxo.scriptPk, this.unisatOpenAPI.getNetwork()),
        safeToSpend: utxoSpendable(utxo),
        confirmation: -1,
      }
    })

    const unspendableUTXOs = Array.from(utxos).map((utxo)=>{
      return {
        n: utxo.vout,
        txid: utxo.txid,
        sats: utxo.satoshis,
        scriptPubKey: decodeScriptPubKey(utxo.scriptPk, this.unisatOpenAPI.getNetwork()),
        safeToSpend: utxoSpendable(utxo),
        confirmation: -1,
      }
    })

    return {
      totalUTXOs: utxos.length,
      spendableUTXOs: spendableUTXOs,
      unspendableUTXOs: unspendableUTXOs
    }
  }

  async relay({ hex }: RelayOptions): Promise<string> {
    return await this.unisatOpenAPI.pushTx(hex)
  }
}

function utxoSpendable(utxo: unisatTypes.UTXO): boolean {
  if (utxo.inscriptions.length>0 || utxo.atomicals.length>0) {
    return false
  }

  return true
}