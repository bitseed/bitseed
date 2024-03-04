import { InscriptionID } from './generator'

export type Tick = {
  tick: string
  max: number
  generator: InscriptionID
  repeat: number
  has_user_input: boolean
  deploy_args: Array<string>
}
