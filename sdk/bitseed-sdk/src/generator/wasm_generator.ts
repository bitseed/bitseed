
import { IGenerator } from './interface';
import { SFT } from '../types';

export class WasmGenerator implements IGenerator {
  private wasmInstance: WebAssembly.Instance;

  constructor(instance: WebAssembly.Instance) {
    this.wasmInstance = instance;
  }

  public async inscribeGenerate(deployArgs: Map<string, string>, seed: string, userInput: string): Promise<SFT> {
    // 将 deployArgs 转换为 JSON 字符串
    const attrs = JSON.stringify(Array.from(deployArgs.entries()));

    // commons
    const mallocFunction = this.wasmInstance.exports.malloc as CallableFunction;
    const freeFunction = this.wasmInstance.exports.free as CallableFunction;
    const inscribeGenerateFunction = this.wasmInstance.exports.inscribe_generate as CallableFunction;

    // 分配内存并写入字符串数据
    const encodeString = (str: string, memory: WebAssembly.Memory) => {
      const encoder = new TextEncoder();
      const encodedString = encoder.encode(str);
      const len = encodedString.length;
      const ptr = mallocFunction(len);

      const dataView = new DataView(memory.buffer);
      for (let i = 0; i < len; i++) {
        dataView.setUint8(ptr + i, encodedString[i]);
      }

      return { ptr, len };
    };

    // 获取 WASM 实例的内存
    const memory = this.wasmInstance.exports.memory as WebAssembly.Memory;

    // 将 seed 和 userInput 编码并写入 WASM 内存
    const seedEncoded = encodeString(seed, memory);
    const userInputEncoded = encodeString(userInput, memory);
    const attrsEncoded = encodeString(attrs, memory);

    // 调用 WASM 函数
    const resultPtr = inscribeGenerateFunction(seedEncoded.ptr, userInputEncoded.ptr, attrsEncoded.ptr);

    // 读取 WASM 内存中的结果字符串
    const decodeString = (ptr: number, memory: WebAssembly.Memory) => {
      const decoder = new TextDecoder();
      let length = 0;
      let currentByte = 0;
      const dataView = new DataView(memory.buffer);
      while ((ptr + length) < dataView.byteLength && (currentByte = dataView.getUint8(ptr + length), currentByte !== 0)) {
        length++;
      }
      const encodedResult = new Uint8Array(memory.buffer, ptr, length);
      return decoder.decode(encodedResult);
    };

    const result = decodeString(resultPtr, memory);

    // 释放 WASM 内存中的字符串数据
    freeFunction(seedEncoded.ptr);
    freeFunction(userInputEncoded.ptr);
    freeFunction(attrsEncoded.ptr);

    // 将结果字符串转换为 JSON 对象
    const sft: SFT = JSON.parse(result);

    // 返回 SFT 对象
    return sft;
  }

  public static async loadWasmModule(wasmBytes: BufferSource): Promise<WasmGenerator> {
    const module = await WebAssembly.compile(wasmBytes);

    const imports = {
      env: {

      }
    };

    const instance = await WebAssembly.instantiate(module, imports);

    return new WasmGenerator(instance);
  }

}
