import { Generator, InscriptionID } from "../types";

export interface DeployOptions {
  repeat?: string;
  has_user_input?: boolean;
  deploy_args?: Map<string, string>;
}

export interface APIInterface {
  name(): string;
  deploy(tick: string, max: number, generator: Generator, opts?: DeployOptions): Promise<InscriptionID>;
  mint(tick: string, amt: number, attributes?: Map<string, string>): Promise<InscriptionID>;
  merge(a: InscriptionID, b: InscriptionID): Promise<InscriptionID>;
  split(a: InscriptionID): Promise<[InscriptionID, InscriptionID]>;
}
