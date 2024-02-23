import { InscriptionID, SFT, DeployArg } from '../types'

export interface IGenerator {
  inscribeGenerate(deployArgs: Array<DeployArg>, seed: string, userInput: string): Promise<SFT>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
