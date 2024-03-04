import * as bitcoin from 'bitcoinjs-lib'
import { BitSeed } from './bitseed';
import { Ordit, IDatasource, RelayOptions} from '@sadoprotocol/ordit-sdk';
import { IGeneratorLoader } from './generator';
import { SFTRecord, InscriptionID } from './types';

const networkType = 'testnet'

describe('BitSeed', () => {
  let primaryWallet: Ordit;
  let fundingWallet: Ordit;
  let datasourceMock: jest.Mocked<IDatasource>;
  let generatorLoaderMock: jest.Mocked<IGeneratorLoader>;
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
    
    generatorLoaderMock = {
      load: jest.fn()
    };

    bitSeed = new BitSeed(
      primaryWallet,
      fundingWallet,
      datasourceMock,
      generatorLoaderMock
    );
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
      datasourceMock.relay.mockImplementation(function ({ hex }: RelayOptions): Promise<string> {
        const tx = bitcoin.Transaction.fromHex(hex)
        const txid = tx.getId()

        return new Promise<string>(function(resolve){
          resolve(txid)
        })
      })

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
});
