import { InscriptionID, SFTRecord, DeployArg } from '../types'

export interface IGenerator {
  inscribeGenerate(deployArgs: Array<DeployArg>, seed: string, userInput: string): Promise<SFTRecord>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
