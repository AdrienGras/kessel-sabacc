// Lazy WASM module singleton
type WasmModule = typeof import("../wasm-pkg/sabacc_wasm");
let wasmModule: WasmModule | null = null;

export async function initWasm(): Promise<WasmModule> {
  if (wasmModule) return wasmModule;

  const mod = await import("../wasm-pkg/sabacc_wasm");
  await mod.default();
  wasmModule = mod;
  return mod;
}

export function getWasm(): WasmModule {
  if (!wasmModule)
    throw new Error("WASM not initialized — call initWasm() first");
  return wasmModule;
}
