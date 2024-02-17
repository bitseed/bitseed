import { BitSeedAsset } from "../types";
import { RoochBitSeedApiInterface } from "./rooch-bitseed-api.interface";

export class BitSeedApiMock implements RoochBitSeedApiInterface {
  getBitSeedAssetByID(): Promise<BitSeedAsset> {
    throw new Error("Method not implemented.");
  }
}