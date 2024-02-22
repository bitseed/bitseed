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
    const deployArgs = new Map<string, string>([['key1', 'value1']]);
    const seed = 'testSeed';
    const userInput = 'testUserInput';

    // 调用inscribeGenerate方法
    const result = await generator.inscribeGenerate(deployArgs, seed, userInput);
    expect(result).toBe({})
  });
});
