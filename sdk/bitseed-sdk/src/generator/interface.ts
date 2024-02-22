import { InscriptionID, SFT } from "../types";

export interface IGenerator {
  inscribeGenerate(deployArgs: Map<string, string>, seed: string, userInput: string): Promise<SFT>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
