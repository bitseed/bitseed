import path from 'path'
import fs from 'fs';
import * as bitcoin from 'bitcoinjs-lib'
import { BitSeed } from './bitseed';
import { 
  Ordit, 
  IDatasource, 
  RelayOptions, 
  GetSpendablesOptions, 
  UTXOLimited,
  GetInscriptionOptions,
  Inscription
} from '@sadoprotocol/ordit-sdk';
import { IGeneratorLoader, GeneratorLoader } from './generator';
import { SFTRecord, InscriptionID } from './types';
import { InscribeOptions, DeployOptions } from './interfaces'
import { toB64 } from './utils';

const networkType = 'testnet'

const loadWasmBytesFromFile = (url: string) => {
  const filePath = path.resolve(url);
  const fileBuffer = fs.readFileSync(filePath);
  return new Uint8Array(fileBuffer)
}

describe('BitSeed', () => {
  const mempool = new Map<string, UTXOLimited[]>();
  const inscriptionStore = new Map<string, Inscription>();

  let primaryWallet: Ordit;
  let fundingWallet: Ordit;
  let datasourceMock: jest.Mocked<IDatasource>;
  let generatorLoaderMock: IGeneratorLoader;
  let bitSeed: BitSeed;

  beforeEach(() => {
    // address: tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h
    primaryWallet = new Ordit({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: networkType,
      type: 'taproot',
    })

    // address: tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv
    fundingWallet = new Ordit({
      wif: 'cNfgnR9UB1garDrQ3WVaQ2LbG4CPxpuEepor44yyuiB8wtSa3Bta',
      network: networkType,
      type: 'taproot',
    })

    mempool.set('tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv', [
      {
        n: 1,
        txid: '9e71d06045d5d677799a70647c9e5484b232aa684b73334038a447c044dc24cd',
        sats: 45465,
        scriptPubKey: {
          asm: 'OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          desc: 'Script witness_v1_taproot',
          hex: '5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv',
          type: 'witness_v1_taproot'
        }
      },
      {
        n: 0,
        txid: '814ccf5c6de163a83081a0f51c42b1c436a2cc8c3303b5d25f91b6efbd50ef3b',
        sats: 10000,
        scriptPubKey: {
          asm: 'OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          desc: 'Script witness_v1_taproot',
          hex: '5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv',
          type: 'witness_v1_taproot'
        }
      }
    ])

    // set simple generator inscription
    let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/generator.wasm'))
    inscriptionStore.set('6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0', {
      id: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0',
      outpoint: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8:0',
      owner: 'tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h',
      genesis: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8',
      fee: -1,
      height: 2580530,
      number: 2174609,
      sat: 600,
      timestamp: 1709590640,
      mediaType: 'application/wasm',
      mediaSize: 45904,
      mediaContent: toB64(wasmBytes),
      value: 600,
      meta: {}
    });

    // set move tick inscription
    inscriptionStore.set('75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0', {
      id: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436i0',
      outpoint: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436:0',
      owner: 'tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h',
      genesis: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436',
      fee: -1,
      height: 2580531,
      number: 2174829,
      sat: 600,
      timestamp: 1709591745,
      mediaType: '',
      mediaSize: 0,
      mediaContent: '',
      value: 600,
      meta: {
        op: 'deploy',
        tick: 'move',
        amount: 1000,
        attributes: {
          repeat: 1,
          generator: '/inscription/6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8i0',
          deploy_args: [`{"height":{"type":"range","data":{"min":1,"max":1000}}}`]
        }
      }
    });

    datasourceMock = {
      getBalance: jest.fn(),
      getInscription: jest.fn(),
      getInscriptionUTXO: jest.fn(),
      getInscriptions: jest.fn(),
      getTransaction: jest.fn(),
      getSpendables: jest.fn(),
      getUnspents: jest.fn(),
      relay: jest.fn()
    };
    
    generatorLoaderMock = new GeneratorLoader(datasourceMock)

    bitSeed = new BitSeed(
      primaryWallet,
      fundingWallet,
      datasourceMock,
      generatorLoaderMock
    );

    datasourceMock.getSpendables.mockImplementation(async function (opts: GetSpendablesOptions): Promise<UTXOLimited[]> {
      let utxos = (mempool.get(opts.address) || new Array<UTXOLimited>()).
        filter((utxo)=>utxo.sats >= opts.value)

      return new Promise<UTXOLimited[]>(function(resolve){
        resolve(utxos)
      })
    })

    datasourceMock.relay.mockImplementation(async function ({ hex }: RelayOptions): Promise<string> {
      const tx = bitcoin.Transaction.fromHex(hex)
      const txid = tx.getId()

      return new Promise<string>(function(resolve){
        setTimeout(()=>{
          resolve(txid)
        }, 10)
      })
    })
  });

  describe('inscribe method', () => {
    it('should throw an error if no address is selected in the primary wallet', async () => {
      primaryWallet.selectedAddress = undefined;

      const sftRecord: SFTRecord = {
        op: 'test',
        tick: 'testTick',
        amount: 1,
        attributes: {}
      };

      await expect(bitSeed.inscribe(sftRecord)).rejects.toThrow('not selected address');
    });


    it('should deposit reveal fee and inscribe successfully', async () => {
      function stringBody(str: string) {
        const encoder = new TextEncoder();
        return encoder.encode(str);
      }

      const sftRecord: SFTRecord = {
        op: 'test',
        tick: 'testTick',
        amount: 1,
        attributes: {},
        content: {
          content_type: 'text/plain',
          body: stringBody('Hello, World!')
        }
      };

      const inscriptionID: InscriptionID = await bitSeed.inscribe(sftRecord);

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('generator method', () => {


    it('should be ok when mint invalid-generator.wasm', async () => {
      let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/invalid-generator.wasm'))
      console.log('wasm length:', wasmBytes.length)

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
      }

      const inscriptionID = await bitSeed.generator("simple", wasmBytes, inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });

    it('should be ok when mint generator.wasm', async () => {
      let wasmBytes = loadWasmBytesFromFile(path.resolve(__dirname, '../tests/data/generator.wasm'))
      console.log('wasm length:', wasmBytes.length)

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
      }

      const inscriptionID = await bitSeed.generator("simple", wasmBytes, inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('deploy method', () => {
    it('deploy move tick should be ok', async () => {
      const tick = 'move';
      const max = 1000;
      const generator = {
        txid: '6f55475ce65054aa8371d618d217da8c9a764cecdaf4debcbce8d6312fe6b4d8',
        index: 0,
      }

      const deployArgs = [
        '{"height":{"type":"range","data":{"min":1,"max":1000}}}'
      ];

      const deployOptions: DeployOptions = {
        fee_rate: 1,
        repeat: 1,
        deploy_args: deployArgs,
      }

      const inscriptionID = await bitSeed.deploy(tick, max, generator, deployOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });

  describe('mint method', () => {

    beforeEach(() => {
      datasourceMock.getInscription.mockImplementation(async function({ id }: GetInscriptionOptions): Promise<Inscription>{
        let inscription = inscriptionStore.get(id)
  
        return new Promise<Inscription>(function(resolve, reject){
          if (!inscription) {
            reject('inscription not exists')
            return
          }

          resolve(inscription)
        })
      })
    })

    it('mint move tick should be ok', async () => {
      const tickInscriptionId = {
        txid: '75e95eeba0b3450feda8d880efe00600816e5934160a4757fbdaa99a0e3bb436',
        index: 0,
      }

      const inscribeOptions: InscribeOptions = {
        fee_rate: 1,
        satpoint: 'xxx'
      }

      const inscriptionID = await bitSeed.mint(tickInscriptionId, 'xxxx', inscribeOptions)

      expect(inscriptionID).toHaveProperty('txid');
      expect(inscriptionID.index).toEqual(0);
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });
});

