import { InscriptionID, SFT } from "../types";

export interface IGenerator {
  inscribeGenerate(deployArgs: Map<string, string>, seed: string, userInput: string): SFT
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
