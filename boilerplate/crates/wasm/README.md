# project-wasm

WebAssembly bindings for `project-core`, built with [wasm-pack](https://rustwasm.github.io/wasm-pack/).

---

## Build

### Prerequisites

```sh
# Install wasm-pack (one-time)
cargo install wasm-pack

# Optional: install wasm-opt for additional size reduction
# macOS:
brew install binaryen
# Ubuntu / Debian:
apt install binaryen
```

### Build for the web

```sh
# From this directory
wasm-pack build --target web --out-dir pkg/

# Or use the helper script from the workspace root
bash scripts/build-wasm.sh
```

The `pkg/` directory will contain:

| File | Purpose |
|---|---|
| `project_wasm_bg.wasm` | Compiled WebAssembly binary |
| `project_wasm.js` | ES module JS glue |
| `project_wasm.d.ts` | TypeScript type declarations |
| `package.json` | npm package metadata |

### Build targets

| Target | Command | Use case |
|---|---|---|
| Web (ES modules) | `wasm-pack build --target web` | Browsers, Vite, Webpack |
| Node.js | `wasm-pack build --target nodejs` | Node scripts, Jest |
| Bundler | `wasm-pack build --target bundler` | Rollup, esbuild |
| No-modules | `wasm-pack build --target no-modules` | Plain `<script>` tags |

---

## Use in JavaScript / TypeScript

### Browser (ES module)

```html
<script type="module">
  import init, { get_version, analyze, process_batch }
    from "./pkg/project_wasm.js";

  await init(); // instantiates the WASM module

  // Version
  console.log(get_version()); // "0.1.0"

  // Analyze a dataset
  const result = JSON.parse(
    analyze(JSON.stringify({ data: [1, 2, 3, 4, 5] }))
  );
  console.log(result);
  // { count: 5, sum: 15, mean: 3, min: 1, max: 5 }

  // Process a batch
  const batch = JSON.parse(
    process_batch(JSON.stringify([
      { id: "a", value: 10 },
      { id: "b", value: 20 },
    ]))
  );
  console.log(batch);
  // { processed: 2, results: [{ id: "a", output: 20, status: "ok" }, ...] }
</script>
```

### Node.js

```js
const { get_version, analyze, process_batch } =
  require("./pkg/project_wasm.js"); // synchronous require for Node target

console.log(get_version());
```

### TypeScript type hints

After building, the generated `pkg/project_wasm.d.ts` exposes:

```ts
/** Returns the crate version string. */
export function get_version(): string;

/**
 * Analyse a dataset.
 * @param input_json JSON string: `{ data: number[] }`
 * @returns JSON string: `{ count, sum, mean, min, max }`
 * @throws {Error} on malformed input or empty data
 */
export function analyze(input_json: string): string;

/**
 * Process a batch of items.
 * @param items_json JSON string: `Array<{ id: string, value: number }>`
 * @returns JSON string: `{ processed: number, results: BatchResult[] }`
 */
export function process_batch(items_json: string): string;
```

---

## JSON API Contract

All public functions use a strict JSON-in / JSON-out convention.  Errors are
thrown as JavaScript `Error` objects with a human-readable message.

### `get_version() -> string`

No input.  Returns the semver string from `Cargo.toml`.

---

### `analyze(input_json: string) -> string`

**Input**

```json
{
  "data": [1.0, 2.5, 3.7, 4.2, 5.1]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `data` | `number[]` | yes | Non-empty array of finite floats |

**Output**

```json
{
  "count": 5,
  "sum": 16.5,
  "mean": 3.3,
  "min": 1.0,
  "max": 5.1
}
```

**Errors**

| Condition | Message |
|---|---|
| `data` is empty | `"input data is empty — at least one element is required"` |
| JSON is malformed | serde parse error message |

---

### `process_batch(items_json: string) -> string`

**Input**

```json
[
  { "id": "item-1", "value": 10.0 },
  { "id": "item-2", "value": 3.14 }
]
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | yes | Caller-supplied identifier, echoed back |
| `value` | `number` | yes | Numeric payload |

**Output**

```json
{
  "processed": 2,
  "results": [
    { "id": "item-1", "output": 20.0,  "status": "ok" },
    { "id": "item-2", "output": 6.28,  "status": "ok" }
  ]
}
```

Per-item `status` is `"ok"` for valid inputs; `"invalid: value is NaN"` (or
similar) for non-finite floats.  An empty array input is valid and returns
`{ "processed": 0, "results": [] }`.

---

## Running Tests

```sh
# Pure Rust unit tests (no browser required)
cargo test

# WASM tests in a headless browser
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome

# WASM tests in Node (no browser required)
wasm-pack test --node
```

---

## Arena Allocator

The `arena` module provides a reusable `Arena<T>` for building in-WASM object
graphs without `Box` or `Rc`.  It is compiled into the rlib for Rust consumers
and is available (via `pub mod arena`) but not directly exposed to JS.

```rust
use project_wasm::arena::{Arena, NodeId};

let mut arena: Arena<String> = Arena::new();
let id: NodeId = arena.alloc("hello".to_owned());
assert_eq!(arena.get(id), Some(&"hello".to_owned()));
```

---

## Release Size

With the aggressive release profile (`opt-level="z"`, `lto`, `strip`,
`panic="abort"`), the `.wasm` output is typically 50-150 KB before `wasm-opt`
and 20-80 KB after (`-Oz`).  Run `scripts/build-wasm.sh` to apply both steps.
