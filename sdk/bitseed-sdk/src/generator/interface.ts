import { InscriptionID, SFTRecord } from '../types'

export interface IGenerator {
  inscribeGenerate(deployArgs: Array<string>, seed: string, userInput: string): Promise<SFTRecord>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
