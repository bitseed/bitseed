import { Generator, InscriptionID } from "../types";

// Define custom types that would be used in the interface
type SatPoint = any; // Replace 'any' with the actual type definition
type FeeRate = number; // Assuming FeeRate is a number representing sats/vbyte
type Amount = number; // Assuming Amount is a number representing satoshis

export interface NestedObject {
  [key: string]: NestedObject | any;
}

// InscribeOptions interface
export interface InscribeOptions {
    /**
     * Inscribe <SATPOINT>. This SatPoint will be used as mint seed.
     */
    satpoint?: SatPoint;

    /**
     * Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.
     * Defaults to <FEE_RATE> if unset.
     */
    commit_fee_rate?: FeeRate;

    /**
     * Send inscription to <DESTINATION>.
     */
    destination?: string;

    /**
     * Don't sign or broadcast transactions.
     */
    dry_run?: boolean;

    /**
     * Use fee rate of <FEE_RATE> sats/vB.
     */
    fee_rate: FeeRate;

    /**
     * Amount of postage to include in the inscription. Default `10000sat`.
     */
    postage?: Amount;

    /**
     * meta for Inscribe
     */
    meta?: NestedObject
}


export interface DeployOptions extends InscribeOptions {
  repeat?: string;
  has_user_input?: boolean;
  deploy_args?: Map<string, string>;
}

export interface APIInterface {
  name(): string;
  generator(wasmBytes: Uint8Array, opts?: InscribeOptions): Promise<InscriptionID>;
  deploy(tick: string, max: number, generator: Generator, opts?: DeployOptions): Promise<InscriptionID>;
  mint(tickInscriptionId: InscriptionID, userInput: string, opts?: InscribeOptions): Promise<InscriptionID>;
  merge(a: InscriptionID, b: InscriptionID): Promise<InscriptionID>;
  split(a: InscriptionID): Promise<[InscriptionID, InscriptionID]>;
}
