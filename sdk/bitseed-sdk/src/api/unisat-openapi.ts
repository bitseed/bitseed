import * as bitcoin from 'bitcoinjs-lib';
import randomstring from 'randomstring';

import { CHANNEL, OPENAPI_URL_MAINNET, OPENAPI_URL_TESTNET, OPENAPI_URL_REGTEST, VERSION } from './unisat-openapi.constants';
import {
  AddressSummary,
  AddressTokenSummary,
  AppSummary,
  Arc20Balance,
  BitcoinBalance,
  DecodedPsbt,
  FeeSummary,
  InscribeOrder,
  Inscription,
  InscriptionSummary,
  TokenBalance,
  TokenTransfer,
  UTXO,
  UTXO_Detail,
  VersionDetail,
  WalletConfig,
  Transaction
} from './unisat-openapi.types';

import { IUniSatOpenAPI } from './unisat-openapi.interface'

interface OpenApiStore {
  host: string;
  deviceId: string;
  config?: WalletConfig;
}

enum API_STATUS {
  FAILED = -1,
  SUCCESS = 0
}

export class UnisatOpenApi implements IUniSatOpenAPI {
  store!: OpenApiStore;
  network: bitcoin.Network
  clientAddress = '';
  addressFlag = 0;

  constructor(network: bitcoin.Network) {
    this.network = network

    this.store = {
      host: OPENAPI_URL_MAINNET,
      deviceId: randomstring.generate(12)
    };

    const networkType = this.network;
    if (networkType === bitcoin.networks.regtest) {
      this.store.host = OPENAPI_URL_REGTEST;
    } else if (networkType === bitcoin.networks.testnet){
      this.store.host = OPENAPI_URL_TESTNET;
    } else {
      this.store.host = OPENAPI_URL_MAINNET;
    }
  }

  getHost() {
    return this.store.host;
  }

  getNetwork(): bitcoin.Network {
    return this.network
  }

  setClientAddress = async (token: string, flag: number) => {
    this.clientAddress = token;
    this.addressFlag = flag;
  };

  getRespData = async (res: any) => {
    let jsonRes: { code: number; msg: string; data: any };

    if (!res) throw new Error('Network error, no response');
    if (res.status !== 200) throw new Error('Network error with status: ' + res.status);
    try {
      jsonRes = await res.json();
    } catch (e) {
      throw new Error('Network error, json parse error');
    }
    if (!jsonRes) throw new Error('Network error,no response data');
    if (jsonRes.code === API_STATUS.FAILED) {
      throw new Error(jsonRes.msg);
    }
    return jsonRes.data;
  };

  httpGet = async (route: string, params: any) => {
    let url = this.getHost() + route;
    let c = 0;
    for (const id in params) {
      if (c == 0) {
        url += '?';
      } else {
        url += '&';
      }
      url += `${id}=${params[id]}`;
      c++;
    }
    const headers = new Headers();
    headers.append('X-Client', 'UniSat Wallet');
    headers.append('X-Version', VERSION);
    headers.append('x-address', this.clientAddress);
    headers.append('x-flag', this.addressFlag + '');
    headers.append('x-channel', CHANNEL);
    headers.append('x-udid', this.store.deviceId);
    let res: Response;
    try {
      res = await fetch(new Request(url), { method: 'GET', headers, mode: 'cors', cache: 'default' });
    } catch (e: any) {
      throw new Error('Network error: ' + e && e.message);
    }

    return this.getRespData(res);
  };

  httpPost = async (route: string, params: any) => {
    const url = this.getHost() + route;
    const headers = new Headers();
    headers.append('X-Client', 'UniSat Wallet');
    headers.append('X-Version', VERSION);
    headers.append('x-address', this.clientAddress);
    headers.append('x-flag', this.addressFlag + '');
    headers.append('x-channel', CHANNEL);
    headers.append('x-udid', this.store.deviceId);
    headers.append('Content-Type', 'application/json;charset=utf-8');
    let res: Response;
    try {
      res = await fetch(new Request(url), {
        method: 'POST',
        headers,
        mode: 'cors',
        cache: 'default',
        body: JSON.stringify(params)
      });
    } catch (e: any) {
      throw new Error('Network error: ' + e && e.message);
    }

    return this.getRespData(res);
  };

  async loadContent(uri: string): Promise<ArrayBuffer> {
    const headers = new Headers();
    headers.append('X-Client', 'UniSat Wallet');
    headers.append('X-Version', VERSION);
    headers.append('x-address', this.clientAddress);
    headers.append('x-flag', this.addressFlag + '');
    headers.append('x-channel', CHANNEL);
    headers.append('x-udid', this.store.deviceId);
    let res: Response;
    try {
      res = await fetch(new Request(uri), { method: 'GET', headers, mode: 'cors', cache: 'default' });
    } catch (e: any) {
      throw new Error('Network error: ' + e && e.message);
    }

    if (!res) throw new Error('Network error, no response');
    if (res.status !== 200) throw new Error('Network error with status: ' + res.status);

    try {
      return await res.arrayBuffer()
    } catch (e) {
      throw new Error('Network error, json parse error');
    }
  }

  async getWalletConfig(): Promise<WalletConfig> {
    return this.httpGet('/default/config', {});
  }

  async getAddressSummary(address: string): Promise<AddressSummary> {
    return this.httpGet('/address/summary', {
      address
    });
  }

  async getAddressBalance(address: string): Promise<BitcoinBalance> {
    return this.httpGet('/address/balance', {
      address
    });
  }

  async getMultiAddressAssets(addresses: string): Promise<AddressSummary[]> {
    return this.httpGet('/address/multi-assets', {
      addresses
    });
  }

  async findGroupAssets(
    groups: { type: number; address_arr: string[] }[]
  ): Promise<{ type: number; address_arr: string[]; satoshis_arr: number[] }[]> {
    return this.httpPost('/address/find-group-assets', {
      groups
    });
  }

  async getBTCUtxos(address: string): Promise<UTXO[]> {
    return this.httpGet('/address/btc-utxo', {
      address
    });
  }

  async getInscriptionUtxo(inscriptionId: string): Promise<UTXO> {
    return this.httpGet('/inscription/utxo', {
      inscriptionId
    });
  }

  async getInscriptionUtxoDetail(inscriptionId: string): Promise<UTXO_Detail> {
    return this.httpGet('/inscription/utxo-detail', {
      inscriptionId
    });
  }

  async getInscriptionUtxos(inscriptionIds: string[]): Promise<UTXO[]> {
    return this.httpPost('/inscription/utxos', {
      inscriptionIds
    });
  }

  async getAddressInscriptions(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }> {
    return this.httpGet('/address/inscriptions', {
      address,
      cursor,
      size
    });
  }

  async getInscriptionSummary(): Promise<InscriptionSummary> {
    return this.httpGet('/default/inscription-summary', {});
  }

  async getAppSummary(): Promise<AppSummary> {
    return this.httpGet('/default/app-summary-v2', {});
  }

  async getTx(txid: string): Promise<Transaction> {
    return this.httpGet('/tx/info', {
      txid
    });
  }

  async pushTx(rawtx: string): Promise<string> {
    return this.httpPost('/tx/broadcast', {
      rawtx
    });
  }

  async getFeeSummary(): Promise<FeeSummary> {
    return this.httpGet('/default/fee-summary', {});
  }

  async getDomainInfo(domain: string): Promise<Inscription> {
    return this.httpGet('/address/search', { domain });
  }

  async inscribeBRC20Transfer(
    address: string,
    tick: string,
    amount: string,
    feeRate: number,
    outputValue: number
  ): Promise<InscribeOrder> {
    return this.httpPost('/brc20/inscribe-transfer', { address, tick, amount, feeRate, outputValue });
  }

  async getInscribeResult(orderId: string): Promise<TokenTransfer> {
    return this.httpGet('/brc20/order-result', { orderId });
  }

  async getBRC20List(address: string, cursor: number, size: number): Promise<{ list: TokenBalance[]; total: number }> {
    return this.httpGet('/brc20/list', { address, cursor, size });
  }

  async getAddressTokenSummary(address: string, ticker: string): Promise<AddressTokenSummary> {
    return this.httpGet('/brc20/token-summary', { address, ticker: encodeURIComponent(ticker) });
  }

  async getTokenTransferableList(
    address: string,
    ticker: string,
    cursor: number,
    size: number
  ): Promise<{ list: TokenTransfer[]; total: number }> {
    return this.httpGet('/brc20/transferable-list', {
      address,
      ticker: encodeURIComponent(ticker),
      cursor,
      size
    });
  }

  async decodePsbt(psbtHex: string): Promise<DecodedPsbt> {
    return this.httpPost('/tx/decode', { psbtHex });
  }

  async createMoonpayUrl(address: string): Promise<string> {
    return this.httpPost('/moonpay/create', { address });
  }

  async checkWebsite(website: string): Promise<{ isScammer: boolean; warning: string }> {
    return this.httpPost('/default/check-website', { website });
  }

  async getOrdinalsInscriptions(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }> {
    return this.httpGet('/ordinals/inscriptions', {
      address,
      cursor,
      size
    });
  }

  async getAtomicalsNFT(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Inscription[]; total: number }> {
    return this.httpGet('/atomicals/nft', {
      address,
      cursor,
      size
    });
  }

  async getAtomicalsUtxo(atomicalId: string): Promise<UTXO> {
    return this.httpGet('/atomicals/utxo', {
      atomicalId
    });
  }

  async getArc20BalanceList(
    address: string,
    cursor: number,
    size: number
  ): Promise<{ list: Arc20Balance[]; total: number }> {
    return this.httpGet('/arc20/balance-list', { address, cursor, size });
  }

  async getArc20Utxos(address: string, ticker: string): Promise<UTXO[]> {
    return this.httpGet('/arc20/utxos', {
      address,
      ticker
    });
  }

  async getVersionDetail(version: string): Promise<VersionDetail> {
    return this.httpGet('/version/detail', {
      version
    });
  }
}
