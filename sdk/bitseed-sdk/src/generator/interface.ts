import { InscriptionID, SFT } from "../types";

export type DeployArgValue = {
  type: string,
  data: any,
}

export type DeployArg = {
  [key: string]: DeployArgValue;
}

export interface IGenerator {
  inscribeGenerate(deployArgs: Array<DeployArg>, seed: string, userInput: string): Promise<SFT>
}

export interface IGeneratorLoader {
  load(inscription_id: InscriptionID): Promise<IGenerator>
}
