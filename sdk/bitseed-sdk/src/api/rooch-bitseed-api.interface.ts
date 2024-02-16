import { BitSeedAsset } from '../types'

export interface RoochBitSeedApiInterface {
  getBitSeedAssetByID(): Promise<BitSeedAsset>
}
