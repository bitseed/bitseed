
import { IGenerator, DeployArg } from './interface';
import { SFT } from '../types';

export class WasmGenerator implements IGenerator {
  private wasmInstance: WebAssembly.Instance;

  constructor(instance: WebAssembly.Instance) {
    this.wasmInstance = instance;
  }

  public async inscribeGenerate(deployArgs: Array<DeployArg>, seed: string, userInput: string): Promise<SFT> {
    // 将 deployArgs 转换为 JSON 字符串
    const attrs = JSON.stringify(deployArgs);

    // 获取 WASM 实例的内存
    const memory = this.wasmInstance.exports.memory as WebAssembly.Memory;

    // 分配内存并写入字符串数据
    const encodeStringOnStack = (str: string, memory: WebAssembly.Memory) => {
      const encoder = new TextEncoder();
      const encodedString = encoder.encode(str + '\0'); // Include null-terminator
      const len = encodedString.length;
      const stackAllocFunction = this.wasmInstance.exports.stackAlloc as CallableFunction;
      const stackSaveFunction = this.wasmInstance.exports.stackSave as CallableFunction;
      const stackRestoreFunction = this.wasmInstance.exports.stackRestore as CallableFunction;
    
      // Save the stack pointer before allocation
      const stackPointer = stackSaveFunction();
    
      // Allocate space on the stack
      const ptr = stackAllocFunction(len);
    
      // Write the string to the stack
      const bytes = new Uint8Array(memory.buffer, ptr, len);
      bytes.set(encodedString);
    
      // Return a function that will restore the stack after use
      return {
        ptr,
        len,
        free: () => stackRestoreFunction(stackPointer)
      };
    };
    

    // 将 seed 和 userInput 编码并写入 WASM 内存
    const seedEncoded = encodeStringOnStack(seed, memory);
    const userInputEncoded = encodeStringOnStack(userInput, memory);
    const attrsEncoded = encodeStringOnStack(attrs, memory);

    // 调用 WASM 函数
    const inscribeGenerateFunction = this.wasmInstance.exports.inscribe_generate as CallableFunction;
    const resultPtr = inscribeGenerateFunction(seedEncoded.ptr, userInputEncoded.ptr, attrsEncoded.ptr);

    // 读取 WASM 内存中的结果字符串
    const decodeString = (ptr: number, memory: WebAssembly.Memory) => {
      const decoder = new TextDecoder();
      const dataView = new DataView(memory.buffer);
      let length = 0;
      while (dataView.getUint8(ptr + length) !== 0) {
        length++;
      }
      const encodedResult = new Uint8Array(memory.buffer, ptr, length);
      return decoder.decode(encodedResult, {
        
      });
    };

    const result = decodeString(resultPtr, memory);
    console.log("result:", result)

    seedEncoded.free();
    userInputEncoded.free();
    attrsEncoded.free();

    // 将结果字符串转换为 JSON 对象
    const sft: SFT = JSON.parse(result);

    // 返回 SFT 对象
    return sft;
  }

  public static async loadWasmModule(wasmBytes: BufferSource): Promise<WasmGenerator> {
    const module = await WebAssembly.compile(wasmBytes);

    const imports = {
      env: {
        memoryBase: 0,
        tableBase: 0,
        memory: new WebAssembly.Memory({ initial: 256 }),
        table: new WebAssembly.Table({ initial: 0, element: 'anyfunc' })
      },
      wasi_snapshot_preview1: {
        fd_write: ()=>{},
        fd_seek: ()=>{},
        fd_close: ()=>{},
        proc_exit: ()=>{}
      }
    };

    const instance = await WebAssembly.instantiate(module, imports);

    return new WasmGenerator(instance);
  }

}
