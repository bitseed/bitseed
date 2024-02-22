
import { InscriptionID } from "./generator"
import { DeployArg } from "./deploy_arg"

export type Tick = {
  tick: string,
  max: number,
  generator: InscriptionID,
  repeat: number,
  has_user_input: boolean,
  deploy_args: Array<DeployArg>
}
