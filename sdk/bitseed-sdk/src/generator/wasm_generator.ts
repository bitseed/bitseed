
import { SFT } from '../types'
import { IGenerator } from "./interface"

export class WasmGenerator implements IGenerator{
  private instance: WebAssembly.Instance;

  constructor(instance: WebAssembly.Instance) {
    this.instance = instance;
  }

  inscribeGenerate(deployArgs: Map<string, string>, seed: string, userInput: string): SFT {
    const generateFunction = this.instance.exports.inscribe_generate as CallableFunction
    const resultJSON = generateFunction(seed, userInput, deployArgs);
    return JSON.parse(resultJSON) as SFT
  }
}