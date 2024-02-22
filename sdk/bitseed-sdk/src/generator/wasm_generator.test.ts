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
    // 创建WasmGenerator实例
    const generator = await loadWasmModuleFromFile(path.resolve(__dirname, '../../tests/data/generator.wasm'))

    // 准备测试数据
    const deployArgs = [
      {
        "amount": {
          type: "range",
          data: {
            min: 1,
            max: 1000,
          }
        }
      }
    ];

    const seed = 'testSeed';
    const userInput = 'testUserInput';

    // 调用inscribeGenerate方法
    const result = await generator.inscribeGenerate(deployArgs, seed, userInput);

    // Assert that result has properties "id" and "amount"
    expect(result).toHaveProperty("id");
    expect(result).toHaveProperty("amount");
  });
});
