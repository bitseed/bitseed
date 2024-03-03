
var out = console.log.bind(console);
var err = console.error.bind(console);

var getHeapMax = () =>
  // Stay one Wasm page short of 4GB: while e.g. Chrome is able to allocate
  // full 4GB Wasm memories, the size will wrap back to 0 bytes in Wasm side
  // for any code that deals with heap sizes, which would require special
  // casing all heap size related code to treat 0 specially.
  2147483648;

export class EmscriptenRuntime {
  public ABORT: boolean = false;
  public EXITSTATUS: number = 0;

  private memory: WebAssembly.Memory;
  private table: WebAssembly.Table;

  constructor() {
    this.memory = new WebAssembly.Memory({ initial: 1024*10 });
    this.table = new WebAssembly.Table({ initial: 0, element: 'anyfunc' });
  }

  getMemory(): WebAssembly.Memory {
    return this.memory
  }

  getTable(): WebAssembly.Table {
    return this.table
  }

  getImports() {
    const wasmImports = {
      env: {
        memoryBase: 0,
        tableBase: 0,
        memory: this.memory,
        table: this.table,
         /** @export */
        __assert_fail: ()=>{
          console.log("assert_fail")
        },
        /** @export */
        __cxa_throw: ()=>{
          console.log("cxa_throw")
        },
        /** @export */
        abort: ()=>{
          console.log("abort")
        },
        /** @export */
        emscripten_memcpy_js: ()=>{
          console.log("emscripten_memcpy_js")
        },
        /** @export */
        emscripten_resize_heap: this.buildResizeHeapFunc(),
        log_string: (offset: number, length: number)=>{
          console.log('log_string offset:', offset, 'length:', length)
          const bytes = new Uint8Array(this.memory.buffer, offset, length);
          const string = new TextDecoder('utf8').decode(bytes);
          out(string);
        }
      },
      wasi_snapshot_preview1: {
        fd_write: () => {console.log('fd_write called')},
        fd_seek: () => {console.log('fd_seek called')},
        fd_close: () => {console.log('fd_close called')},
        proc_exit: () => {console.log('proc_exit called')},
      }
    }

    return wasmImports
  }

  HEAPU8() {
    return new Uint8Array(this.memory.buffer)
  }

  abort(what: any) {
    what = 'Aborted(' + what + ')';
    // TODO(sbc): Should we remove printing and leave it up to whoever
    // catches the exception?
    err(what);
  
    this.ABORT = true;
    this.EXITSTATUS = 1;
  
    // Use a wasm runtime error, because a JS error might be seen as a foreign
    // exception, which means we'd run destructors on it. We need the error to
    // simply make the program stop.
    // FIXME This approach does not work in Wasm EH because it currently does not assume
    // all RuntimeErrors are from traps; it decides whether a RuntimeError is from
    // a trap or not based on a hidden field within the object. So at the moment
    // we don't have a way of throwing a wasm trap from JS. TODO Make a JS API that
    // allows this in the wasm spec.
  
    // Suppress closure compiler warning here. Closure compiler's builtin extern
    // definition for WebAssembly.RuntimeError claims it takes no arguments even
    // though it can.
    // TODO(https://github.com/google/closure-compiler/pull/3913): Remove if/when upstream closure gets fixed.
    /** @suppress {checkTypes} */
    var e = new WebAssembly.RuntimeError(what);
  
    // Throw the error whether or not MODULARIZE is set because abort is used
    // in code paths apart from instantiation where an exception is expected
    // to be thrown when abort is called.
    throw e;
  }

  assert(condition: any, text?: any) {
    if (!condition) {
      this.abort('Assertion failed' + (text ? ': ' + text : ''));
    }
  }

  growMemory(size: any): number {
    var b = this.memory.buffer;
    var pages = (size - b.byteLength + 65535) / 65536;

    try {
      // round size grow request up to wasm page size (fixed 64KB per spec)
      this.memory.grow(pages); // .grow() takes a delta compared to the previous size
      return 1 /*success*/;
    } catch(e) {
      err(`growMemory: Attempted to grow heap from ${b.byteLength} bytes to ${size} bytes, but got error: ${e}`);
      return 0
    }
    // implicit 0 return to save code size (caller will cast "undefined" into 0
    // anyhow)
  };

  buildResizeHeapFunc() {
    return (requestedSize: any) => {
      console.log("emscripten_resize_heap, requestedSize:", requestedSize)

      var oldSize = this.HEAPU8().length;
      // With CAN_ADDRESS_2GB or MEMORY64, pointers are already unsigned.
      requestedSize >>>= 0;
      // With multithreaded builds, races can happen (another thread might increase the size
      // in between), so return a failure, and let the caller retry.
      this.assert(requestedSize > oldSize);
    
      // Memory resize rules:
      // 1.  Always increase heap size to at least the requested size, rounded up
      //     to next page multiple.
      // 2a. If MEMORY_GROWTH_LINEAR_STEP == -1, excessively resize the heap
      //     geometrically: increase the heap size according to
      //     MEMORY_GROWTH_GEOMETRIC_STEP factor (default +20%), At most
      //     overreserve by MEMORY_GROWTH_GEOMETRIC_CAP bytes (default 96MB).
      // 2b. If MEMORY_GROWTH_LINEAR_STEP != -1, excessively resize the heap
      //     linearly: increase the heap size by at least
      //     MEMORY_GROWTH_LINEAR_STEP bytes.
      // 3.  Max size for the heap is capped at 2048MB-WASM_PAGE_SIZE, or by
      //     MAXIMUM_MEMORY, or by ASAN limit, depending on which is smallest
      // 4.  If we were unable to allocate as much memory, it may be due to
      //     over-eager decision to excessively reserve due to (3) above.
      //     Hence if an allocation fails, cut down on the amount of excess
      //     growth, in an attempt to succeed to perform a smaller allocation.
    
      // A limit is set for how much we can grow. We should not exceed that
      // (the wasm binary specifies it, so if we tried, we'd fail anyhow).
      var maxHeapSize = getHeapMax();
      if (requestedSize > maxHeapSize) {
        err(`Cannot enlarge memory, requested ${requestedSize} bytes, but the limit is ${maxHeapSize} bytes!`);
        return false;
      }
    
      var alignUp = (x: any, multiple: any) => x + (multiple - x % multiple) % multiple;
    
      var newSize = 0;

      // Loop through potential heap size increases. If we attempt a too eager
      // reservation that fails, cut down on the attempted size and reserve a
      // smaller bump instead. (max 3 times, chosen somewhat arbitrarily)
      for (var cutDown = 1; cutDown <= 4; cutDown *= 2) {
        var overGrownHeapSize = oldSize * (1 + 0.2 / cutDown); // ensure geometric growth
        // but limit overreserving (default to capping at +96MB overgrowth at most)
        overGrownHeapSize = Math.min(overGrownHeapSize, requestedSize + 100663296 );
    
        newSize = Math.min(maxHeapSize, alignUp(Math.max(requestedSize, overGrownHeapSize), 65536));
    
        var replacement = this.growMemory(newSize);
        if (replacement) {
          console.log('growMemory ok')
          return true;
        }
      }
      err(`Failed to grow the heap from ${oldSize} bytes to ${newSize} bytes, not enough memory!`);
      return false;
    };
  }
}