import { JsonRpcDatasource } from '@sadoprotocol/ordit-sdk'
import { Ordit } from '@sadoprotocol/ordit-sdk'
import { BitSeed, GeneratorLoader } from '../../../src'

const network = 'testnet'
const datasource = new JsonRpcDatasource({ network })
const generatorLoader = new GeneratorLoader(datasource)

export function createTestBitSeed(): BitSeed {
  // address: tb1pz9qq9gwemapvmpntw90ygalhnjzgy2d7tglts0a90avrre902z2sh3ew0h
  const primaryWallet = new Ordit({
    wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
    network,
    type: 'taproot',
  })

  // address: tb1p2lsktn6x2eq5h7wfk50xfrr2hlpjhp7q0gget6p4957hy2swt3jsar6zny
  const fundingWallet = new Ordit({
    wif: 'cTW1Q2A8AVBuJ1sEBoV9gWokc6e5NYFPHxez6hhriVL2jKH6bfct',
    network,
    type: 'taproot',
  })

  console.log('primary wallet address:', primaryWallet.selectedAddress)
  console.log('funding wallet address:', fundingWallet.selectedAddress)

  const bitseed = new BitSeed(primaryWallet, fundingWallet, datasource, generatorLoader)

  return bitseed
}
