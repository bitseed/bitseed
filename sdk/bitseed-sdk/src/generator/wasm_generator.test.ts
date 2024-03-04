import path from 'path'
import fs from 'fs';

import { WasmGenerator } from './wasm_generator';

const loadWasmModuleFromFile = async(url: string) => {
  const filePath = path.resolve(url);
  const fileBuffer = fs.readFileSync(filePath);
  return await WasmGenerator.loadWasmModule(fileBuffer)
}

describe('WasmGenerator', () => {
  it('should call inscribe_generate with correct parameters', async () => {
    // Create an instance of WasmGenerator
    const generator = await loadWasmModuleFromFile(path.resolve(__dirname, '../../tests/data/generator.wasm'))
    //const generator = await loadWasmModuleFromFile(path.resolve(__dirname, '../../../../generator/generator.wasm'))

    // Prepare test data
    const deployArgs = [
      `{"height":{"type":"range","data":{"min":1,"max":1000}}}`
    ];

    const seed = 'testSeed';
    const userInput = 'testUserInput';

    // Call the inscribeGenerate method
    const result = await generator.inscribeGenerate(deployArgs, seed, userInput);
    console.log('result:', result)

    // Assert that result has properties "id" and "amount"
    expect(result).toHaveProperty("amount");
    expect(result).toHaveProperty("attributes");
    expect(result).toHaveProperty("content");
  });
});
