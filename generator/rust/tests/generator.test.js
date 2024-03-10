const fs = require('fs');
const path = require('path');
const cbor = require('cbor');

// 加载 WebAssembly 模块
const wasmPath = path.join(__dirname, '../pkg/generator_bg.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBuffer);
const wasmInstance = new WebAssembly.Instance(wasmModule, {
  env: {
    js_log: (ptr, len) => {
      const message = new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
      console.log(message);
    },
  },
});
const { inscribe_generate, memory } = wasmInstance.exports;

// 编码输入数据并复制到 WebAssembly 内存中
function encodeInput(input) {
  const encodedData = cbor.encode(input);
  console.log('encodedInputData:', JSON.stringify(Array.from(encodedData)))

  const len = encodedData.length;
  const ptr = memory.buffer.byteLength;
  memory.grow(Math.ceil(len / 65536));
  const view = new Uint8Array(memory.buffer, ptr, len);
  view.set(encodedData);
  return [ptr, len];
}

// 解码输出数据
function decodeOutput(ptr, len) {
  const view = new Uint8Array(memory.buffer, ptr, len);
  const decodedData = cbor.decode(view);
  return decodedData;
}

// 测试 inscribe_generate 函数
describe('inscribe_generate', () => {
  test('generates correct output for valid input', () => {
    const deployArgs = [
      '{"level1":{"type":"range","data":{"min":1,"max":1000}}}', 
      '{"level2":{"type":"range","data":{"min":1,"max":1000}}}',
    ]

    const argsBytes = new Uint8Array(cbor.encodeOne(deployArgs.map((json)=>JSON.parse(json))))
    const argsArray = Array.from(argsBytes)
    console.log('argsArray:', JSON.stringify(argsArray))

    const input = {
      attrs: argsArray,
      seed: 'random-seed',
      user_input: 'user-input',
    };

    console.log('input:', input)

    const [inputPtr, inputLen] = encodeInput(input);
    console.log('inputPtr:', inputPtr)
    console.log('inputLen:', inputLen)

    const outputPtr = inscribe_generate(inputPtr, inputLen);
    console.log('outputPtr:', outputPtr)

    const output = decodeOutput(outputPtr, 1024);
    expect(output).toEqual({
      amount: 1000,
      attributes: null,
      content: {
        content_type: 'text/plain',
        content: Uint8Array.from([72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]),
        content_len: 13,
      },
    });
  });
});
