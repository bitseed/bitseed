import * as bitcoin from 'bitcoinjs-lib';
import { UnisatOpenApi } from '../../api/unisat-openapi';
import { UniSatDataSource } from './unisat_datasource'; // 假设你的类名为 YourClass
import { Wallet } from '../../wallet'

describe('UniSatDataSource', () => {
  let instance: UniSatDataSource;
  let wallet: Wallet;

  beforeEach(() => {
    wallet = new Wallet({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: "testnet",
      type: 'taproot',
    })

    const openAPI = new UnisatOpenApi(bitcoin.networks.testnet)
    instance = new UniSatDataSource(openAPI);
  });

  describe('getBalance', () => {
    it('should return the correct balance for a given address', async () => {
      if (!wallet.selectedAddress) {
        throw new Error('no selected address')
      }
  
      const balance = await instance.getBalance({ address: wallet.selectedAddress });
      expect(balance).toBe(0.000021);
    });

    it('should return the 0 balance for a new address', async () => {
      const wallet = new Wallet({
        bip39: 'right second until palace kid wear tennis phone bike broccoli oval saddle',
        network: "testnet",
        type: 'taproot',
      })

      const account = wallet.generateAddress('taproot', 2, 0)
      if (!account.address) {
        throw new Error('no selected address')
      }

      const balance = await instance.getBalance({ address: account.address });
      expect(balance).toBe(0.0);
    });
  });

  describe('getInscriptionUTXO', () => {
    it('should return the correct UTXO for getInscriptionUTXO', async () => {
      const utxo = await instance.getInscriptionUTXO({ id: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5i1' });

      expect(utxo).toBeDefined()
      expect(utxo.n).toBe(0)
      expect(utxo.txid).toBe('42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5')
      expect(utxo.sats).toBe(600)
      expect(utxo.safeToSpend).toBeFalsy()
      expect(utxo.confirmation).toBe(-1)
    });
  });

  describe('getInscription', () => {
    jest.setTimeout(10000)
    
    it('should return the correct Inscription for getInscriptionUTXO', async () => {
      const inscription = await instance.getInscription({ id: '42d186a5d9bc064e5704024afb2dfccd424da1b9756ae31a4fbfee22f4fc7ec5i0', decodeMetadata: true });
      console.log('inscription2:', inscription)
      expect(inscription).toBeDefined()
    });
  });
})
