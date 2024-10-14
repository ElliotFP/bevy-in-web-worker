// Declare a variable to hold the WebAssembly module
let wasm;

// Create a TextDecoder for UTF-8 encoding, with options to ignore BOM and throw on invalid input
const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } });

// If TextDecoder is available, decode an empty input to warm up the decoder
if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

// Cache for the Uint8Array view of the WebAssembly memory
let cachedUint8Memory0 = null;

// Function to get or create a Uint8Array view of the WebAssembly memory
function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

// Function to decode a string from WebAssembly memory
function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

// Create a heap for JavaScript objects
const heap = new Array(128).fill(undefined);

// Add special values to the heap
heap.push(undefined, null, true, false);

// Keep track of the next available slot in the heap
let heap_next = heap.length;

// Function to add an object to the heap and return its index
function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

// Function to get an object from the heap by its index
function getObject(idx) { return heap[idx]; }

// Variable to store the length of a vector in WebAssembly memory
let WASM_VECTOR_LEN = 0;

// Create a TextEncoder for UTF-8 encoding
const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } });

// Function to encode a string into WebAssembly memory
const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
        return cachedTextEncoder.encodeInto(arg, view);
    }
    : function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    });

// Function to pass a string to WebAssembly memory
function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

// Function to check if a value is null or undefined
function isLikeNone(x) {
    return x === undefined || x === null;
}

// Cache for the Int32Array view of the WebAssembly memory
let cachedInt32Memory0 = null;

// Function to get or create an Int32Array view of the WebAssembly memory
function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

// Function to remove an object from the heap
function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

// Function to take an object from the heap and remove it
function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

// Function to create a debug string representation of a value
function debugString(val) {
    // Handle primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // Handle objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for (let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in objects
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // Handle user-defined classes or plain Objects
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // Handle Error objects
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO: Handle other types like Set and Map
    return className;
}

// Create a FinalizationRegistry for managing closures, or a dummy object if not supported
const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => { }, unregister: () => { } }
    : new FinalizationRegistry(state => {
        wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
    });

// Function to create a mutable closure
function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // Increment the internal reference count
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

// Adapter functions for WebAssembly closures
function __wbg_adapter_30(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__haabb65b4aab4cd25(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_33(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h9e3cb26358947ed3(arg0, arg1, addHeapObject(arg2));
}

// Function to throw an error for undefined functions
function notDefined(what) { return () => { throw new Error(`${what} is not defined`); }; }

// Export functions for interacting with the WebAssembly module

// Initialize the Bevy application
export function init_bevy_app() {
    const ret = wasm.init_bevy_app();
    return BigInt.asUintN(64, ret);
}

// Create a window using a canvas element
export function create_window_by_canvas(ptr, canvas_id, scale_factor) {
    const ptr0 = passStringToWasm0(canvas_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.create_window_by_canvas(ptr, ptr0, len0, scale_factor);
}

// Create a window using an offscreen canvas
export function create_window_by_offscreen_canvas(ptr, canvas, scale_factor) {
    wasm.create_window_by_offscreen_canvas(ptr, addHeapObject(canvas), scale_factor);
}

// Check if preparation is completed
export function is_preparation_completed(ptr) {
    const ret = wasm.is_preparation_completed(ptr);
    return ret >>> 0;
}

// Handle mouse movement
export function mouse_move(ptr, x, y) {
    wasm.mouse_move(ptr, x, y);
}

// Handle left mouse button down event
export function left_bt_down(ptr, obj, x, y) {
    wasm.left_bt_down(ptr, addHeapObject(obj), x, y);
}

// Handle left mouse button up event
export function left_bt_up(ptr) {
    wasm.left_bt_up(ptr);
}

// Set hover state
export function set_hover(ptr, arr) {
    wasm.set_hover(ptr, addHeapObject(arr));
}

// Set selection state
export function set_selection(ptr, arr) {
    wasm.set_selection(ptr, addHeapObject(arr));
}

// Set auto animation state
export function set_auto_animation(ptr, needs_animate) {
    wasm.set_auto_animation(ptr, needs_animate);
}

// Enter a new frame
export function enter_frame(ptr) {
    wasm.enter_frame(ptr);
}

// Release the application
export function release_app(ptr) {
    wasm.release_app(ptr);
}

// Function to handle errors in WebAssembly function calls
function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

// Cache for the Uint32Array view of the WebAssembly memory
let cachedUint32Memory0 = null;

// Function to get or create a Uint32Array view of the WebAssembly memory
function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

// Function to get a Uint32Array from WebAssembly memory
function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

// Function to load the WebAssembly module
async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

// Function to get the imports for the WebAssembly module
function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    // ... (many import functions defined here)
    // These functions define the JavaScript interface for the WebAssembly module

    return imports;
}

// Function to initialize the WebAssembly memory
function __wbg_init_memory(imports, maybe_memory) {
    // This function is empty in the provided code
}

// Function to finalize the initialization of the WebAssembly module
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedInt32Memory0 = null;
    cachedUint32Memory0 = null;
    cachedUint8Memory0 = null;

    return wasm;
}

// Function to synchronously initialize the WebAssembly module
function initSync(module) {
    if (wasm !== undefined) return wasm;

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

// Function to asynchronously initialize the WebAssembly module
async function __wbg_init(input) {
    if (wasm !== undefined) return wasm;

    if (typeof input === 'undefined') {
        input = new URL('bevy_in_web_worker_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    imports.wbg.__wbg_log_c9486ca5d8e2cbe8 = function (arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.log(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_log_aba5996d9bde071f = function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
        let deferred0_0;
        let deferred0_1;
        let deferred1_0;
        let deferred1_1;
        let deferred2_0;
        let deferred2_1;
        let deferred3_0;
        let deferred3_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            deferred1_0 = arg2;
            deferred1_1 = arg3;
            deferred2_0 = arg4;
            deferred2_1 = arg5;
            deferred3_0 = arg6;
            deferred3_1 = arg7;
            console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    };
    imports.wbg.__wbg_Window_94d759f1f207a15b = function (arg0) {
        const ret = getObject(arg0).Window;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_WorkerGlobalScope_b13c8cef62388de9 = function (arg0) {
        const ret = getObject(arg0).WorkerGlobalScope;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_undefined = function (arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_number_new = function (arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_new = function (arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_clone_ref = function (arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function (arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function () {
        const ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function (arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbg_error_71d6845bf00a930f = function (arg0, arg1) {
        var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
        wasm.__wbindgen_free(arg0, arg1 * 4);
        console.error(...v0);
    };
    imports.wbg.__wbg_instanceof_Window_9029196b662bc42a = function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Window;
        } catch {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_document_f7ace2b956f30a4f = function (arg0) {
        const ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_navigator_7c9103698acde322 = function (arg0) {
        const ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_innerWidth_ebe07ce5463ff293 = function () {
        return handleError(function (arg0) {
            const ret = getObject(arg0).innerWidth;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_innerHeight_1d72454357556162 = function () {
        return handleError(function (arg0) {
            const ret = getObject(arg0).innerHeight;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_devicePixelRatio_5555555565451250 = function (arg0) {
        const ret = getObject(arg0).devicePixelRatio;
        return ret;
    };
    imports.wbg.__wbg_performance_545121019567771d = function (arg0) {
        const ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_now_52205565495b754f = function (arg0) {
        const ret = getObject(arg0).now();
        return ret;
    };
    imports.wbg.__wbg_requestAnimationFrame_241f03c4f0f0140d = function () {
        return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
            return ret;
        }, arguments)
    };
    imports.wbg.__wbg_cancelAnimationFrame_57101631f114268f = function () {
        return handleError(function (arg0, arg1) {
            getObject(arg0).cancelAnimationFrame(arg1);
        }, arguments)
    };
    imports.wbg.__wbg_matchMedia_5573c92551725555 = function () {
        return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, (arg1 + 4)));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_clearTimeout_5647053612543460 = function (arg0, arg1) {
        getObject(arg0).clearTimeout(arg1);
    };
    imports.wbg.__wbg_setTimeout_465a01556a752482 = function () {
        return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
            return ret;
        }, arguments)
    };
    imports.wbg.__wbg_get_44be0491f933a435 = function (arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_9e1ae1900cb0fbd5 = function (arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_898a68150f225f2e = function () {
        const ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newnoargs_b5b063fc6c2f0376 = function (arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_97ae9d8645dc388b = function () {
        return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_is_40a668427d154349 = function (arg0, arg1) {
        const ret = Object.is(getObject(arg0), getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_self_6d479506f72c6a71 = function () {
        return handleError(function () {
            const ret = self.self;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_window_f2557cc78490aceb = function () {
        return handleError(function () {
            const ret = window.window;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_globalThis_7f206bda628d5286 = function () {
        return handleError(function () {
            const ret = globalThis.globalThis;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbg_global_ba75c50d1cf384f4 = function () {
        return handleError(function () {
            const ret = global.global;
            return addHeapObject(ret);
        }, arguments)
    };
    imports.wbg.__wbindgen_is_object = function (arg0) {
        const val = getObject(arg0);
        const ret = typeof (val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbg_has_6f0091915d773777 = function (arg0, arg1) {
        const ret = getObject(arg0).has(getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_set_bf3f89b92d5a34bf = function () {
        return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments)
    };
    imports.wbg.__wbg_buffer_3f3d764d4747d564 = function (arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_6da8e527659b86aa = function (arg0, arg1, arg2) {
        const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_8c3f0052272a457a = function (arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_83db9690f9353e79 = function (arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_length_9e1ae1900cb0fbd5 = function (arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_instanceof_ArrayBuffer_39ac22089b74fddb = function (arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof ArrayBuffer;
        } catch {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_new_8125e318e6245eed = function (arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_8f070f688b0ab574 = function (arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_length_72e2208bbc0efc61 = function (arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_newwithlength_f5933855e4f48a19 = function (arg0) {
        const ret = new Uint8Array(arg0 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_subarray_58ad4efbb5bcb886 = function (arg0, arg1, arg2) {
        const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_969ad0a60e41bd3e = function () {
        return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments)
    };
    imports.wbg.__wbindgen_debug_string = function (arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len1;
        getInt32Memory0()[arg0 / 4 + 0] = ptr1;
    };
    imports.wbg.__wbindgen_throw = function (arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function () {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper127 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 127, __wbg_adapter_30);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper129 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 129, __wbg_adapter_33);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper131 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 131, __wbg_adapter_36);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper133 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 133, __wbg_adapter_39);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper135 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 135, __wbg_adapter_42);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper137 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 137, __wbg_adapter_45);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper139 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 139, __wbg_adapter_48);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper141 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 141, __wbg_adapter_51);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper143 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 143, __wbg_adapter_54);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper145 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 145, __wbg_adapter_57);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper147 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 147, __wbg_adapter_60);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper149 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 149, __wbg_adapter_63);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper151 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 151, __wbg_adapter_66);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper153 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 153, __wbg_adapter_69);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper155 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 155, __wbg_adapter_72);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper157 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 157, __wbg_adapter_75);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper159 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 159, __wbg_adapter_78);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper161 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 161, __wbg_adapter_81);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper163 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 163, __wbg_adapter_84);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper165 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 165, __wbg_adapter_87);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper167 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 167, __wbg_adapter_90);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper169 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 169, __wbg_adapter_93);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper171 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 171, __wbg_adapter_96);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper173 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 173, __wbg_adapter_99);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper175 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 175, __wbg_adapter_102);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper177 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 177, __wbg_adapter_105);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper179 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 179, __wbg_adapter_108);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper181 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 181, __wbg_adapter_111);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper183 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 183, __wbg_adapter_114);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper185 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 185, __wbg_adapter_117);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper187 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 187, __wbg_adapter_120);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper189 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 189, __wbg_adapter_123);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper191 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 191, __wbg_adapter_126);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper193 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 193, __wbg_adapter_129);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper195 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 195, __wbg_adapter_132);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper197 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 197, __wbg_adapter_135);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper199 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 199, __wbg_adapter_138);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper201 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 201, __wbg_adapter_141);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper203 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 203, __wbg_adapter_144);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper205 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 205, __wbg_adapter_147);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper207 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 207, __wbg_adapter_150);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper209 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 209, __wbg_adapter_153);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper211 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 211, __wbg_adapter_156);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper213 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 213, __wbg_adapter_159);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper215 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 215, __wbg_adapter_162);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper217 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 217, __wbg_adapter_165);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper219 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 219, __wbg_adapter_168);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper221 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 221, __wbg_adapter_171);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper223 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 223, __wbg_adapter_174);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper225 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 225, __wbg_adapter_177);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper227 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 227, __wbg_adapter_180);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper229 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 229, __wbg_adapter_183);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper231 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 231, __wbg_adapter_186);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper233 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 233, __wbg_adapter_189);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper235 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 235, __wbg_adapter_192);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper237 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 237, __wbg_adapter_195);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper239 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 239, __wbg_adapter_198);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper241 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 241, __wbg_adapter_201);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper243 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 243, __wbg_adapter_204);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper245 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 245, __wbg_adapter_207);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper247 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 247, __wbg_adapter_210);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper249 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 249, __wbg_adapter_213);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper251 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 251, __wbg_adapter_216);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper253 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 253, __wbg_adapter_219);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper255 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 255, __wbg_adapter_222);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper257 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 257, __wbg_adapter_225);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper259 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 259, __wbg_adapter_228);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper261 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 261, __wbg_adapter_231);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper263 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 263, __wbg_adapter_234);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper265 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 265, __wbg_adapter_237);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper267 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 267, __wbg_adapter_240);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper269 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 269, __wbg_adapter_243);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper271 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 271, __wbg_adapter_246);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper273 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 273, __wbg_adapter_249);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper275 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 275, __wbg_adapter_252);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper277 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 277, __wbg_adapter_255);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper279 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 279, __wbg_adapter_258);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper281 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 281, __wbg_adapter_261);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper283 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 283, __wbg_adapter_264);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper285 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 285, __wbg_adapter_267);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper287 = function (arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 287, __wbg_adapter_270);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper289 = function (arg0, arg1, arg2) {

        return imports;
    }

    function __wbg_init_memory(imports, maybe_memory) {

    }

    function __wbg_finalize_init(instance, module) {
        wasm = instance.exports;
        __wbg_init.__wbindgen_wasm_module = module;
        cachedInt32Memory0 = null;
        cachedUint32Memory0 = null;
        cachedUint8Memory0 = null;


        return wasm;
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;

        const imports = __wbg_get_imports();

        __wbg_init_memory(imports);

        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }

        const instance = new WebAssembly.Instance(module, imports);

        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(input) {
        if (wasm !== undefined) return wasm;

        if (typeof input === 'undefined') {
            input = new URL('bevy_in_web_worker_bg.wasm', import.meta.url);
        }
        const imports = __wbg_get_imports();

        if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
            input = fetch(input);
        }

        __wbg_init_memory(imports);

        const { instance, module } = await __wbg_load(await input, imports);

        return __wbg_finalize_init(instance, module);
    }

    export { initSync }
    export default __wbg_init;
