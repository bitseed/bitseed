import { Transaction as BTCTransaction } from "bitcoinjs-lib";
import { SATOSHIS_PER_BTC } from '../../constants'
import { GetBalanceOptions, GetInscriptionOptions, GetInscriptionsOptions, GetInscriptionUTXOOptions, GetSpendablesOptions, GetTransactionOptions, GetUnspentsOptions, GetUnspentsResponse, IDatasource, Inscription, RelayOptions, Transaction, UTXO, UTXOLimited } from "@sadoprotocol/ordit-sdk";
import { IUniSatOpenAPI } from "../../api";
import { decodeScriptPubKey } from '../../utils/bitcoin';

export class UniSatDataSource implements IDatasource {
  private unisatOpenAPI: IUniSatOpenAPI

  constructor(unisatOpenAPI: IUniSatOpenAPI) {
    this.unisatOpenAPI = unisatOpenAPI;
  }

  async getBalance({ address }: GetBalanceOptions): Promise<number> {
    const balance = await this.unisatOpenAPI.getAddressBalance(address);
    
    const balanceBigInt = BigInt(balance.amount);
    const dividedBalance = balanceBigInt / SATOSHIS_PER_BTC;
    const balanceNumber = Number(dividedBalance);
    
    if (!Number.isSafeInteger(balanceNumber)) {
      throw new Error('Balance is too large to represent as a number safely.');
    }
    
    return balanceNumber;
  }
  
  async getInscriptionUTXO({ id }: GetInscriptionUTXOOptions): Promise<UTXO> {
    const utxo = await this.unisatOpenAPI.getInscriptionUtxo(id);

    return {
      n: utxo.vout,
      txid: utxo.txid,
      sats: utxo.satoshis,
      scriptPubKey: decodeScriptPubKey(utxo.scriptPk, this.unisatOpenAPI.getNetwork()),
      safeToSpend: utxo.inscriptions.length == 0,
      confirmation: 1,
    }
  }

  async getInscription({ id, decodeMetadata }: GetInscriptionOptions): Promise<Inscription> {
    const utxoDetail = await this.unisatOpenAPI.getInscriptionUtxoDetail(id);

    if (!utxoDetail || utxoDetail.inscriptions.length == 0) {
      throw new Error('inscription nil')
    }

    if (utxoDetail && utxoDetail.inscriptions.length > 2) {
      throw new Error('more than one Inscription')
    }

    const inscription = utxoDetail.inscriptions[0]
    return {
      id: inscription.inscriptionId,
      outpoint: inscription.output,
      owner: inscription.address,
      genesis: inscription.genesisTransaction,
      fee: 0,
      height: inscription.utxoHeight,
      number: inscription.inscriptionNumber,
      sat: utxoDetail.satoshis,
      timestamp: inscription.timestamp,
      mediaType: inscription.contentType,
      mediaSize: inscription.contentLength,
      mediaContent: inscription.contentBody,
      value: inscription.outputValue,
    }
  }

  async getInscriptions({ owner, limit, next }: GetInscriptionsOptions): Promise<Inscription[]> {
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
        fee: 0,
        height: inscription.utxoHeight,
        number: inscription.inscriptionNumber,
        sat: 0,
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
  
  getSpendables({ address, value, type, rarity, filter, limit }: GetSpendablesOptions): Promise<UTXOLimited[]> {
    throw new Error("Method not implemented.");
  }
  getUnspents({ address, type, rarity, sort, limit, next }: GetUnspentsOptions): Promise<GetUnspentsResponse> {
    throw new Error("Method not implemented.");
  }

  async relay({ hex }: RelayOptions): Promise<string> {
    return await this.unisatOpenAPI.pushTx(hex)
  }
}