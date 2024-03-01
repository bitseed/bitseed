import { BitSeed } from './bitseed';
import { Ordit, IDatasource, UTXOLimited, GetSpendablesOptions} from '@sadoprotocol/ordit-sdk';
import { IGeneratorLoader } from './generator';
import { SFTRecord, InscriptionID } from './types';
 
const network = 'testnet'

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
      network,
      type: 'taproot',
    })

    // address: tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv
    fundingWallet = new Ordit({
      wif: 'cNfgnR9UB1garDrQ3WVaQ2LbG4CPxpuEepor44yyuiB8wtSa3Bta',
      network,
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
      const utxos = new Array<UTXOLimited>();
      utxos.push({
        n: 1,
        txid: 'f2e6e08f7ddd3ce0dfaaf5e7d8a7709948539582347f8d55a5a53c3961519087',
        sats: 52153,
        scriptPubKey: {
          asm: 'OP_1 b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          desc: 'Script witness_v1_taproot',
          hex: '5120b69d4d0bbf765eb327eecb59a6d76280b958c6b9bac195109261f6a44746fa07',
          address: 'tb1pk6w56zalwe0txflwedv6d4mzszu4334ehtqe2yyjv8m2g36xlgrs7m68qv',
          type: 'witness_v1_taproot'
        }
      })

      datasourceMock.getSpendables.mockImplementation(function (_opts: GetSpendablesOptions): Promise<UTXOLimited[]> {
        return new Promise<UTXOLimited[]>(function(resolve){
          resolve(utxos)
        })
      })

      datasourceMock.relay.mockResolvedValueOnce('depositTxId').mockResolvedValueOnce('inscribeTxId');

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

      expect(inscriptionID).toEqual({ txid: 'inscribeTxId', index: 0 });
      expect(datasourceMock.relay).toHaveBeenCalledTimes(2);
    });
  });
});

