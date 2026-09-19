# HTTP + Key-Value Handler in Rust

This project template is a WebAssembly component built with [Rust][rust] that stores and retrieves key-value pairs over HTTP, backed by [`wasi:keyvalue`][wasi-kv].

The component speaks only the `wasi:keyvalue/store` interface. The host runtime selects the underlying storage backend (in-memory, filesystem, NATS, Redis, and others) based on `.wash/config.yaml`, so the same component code runs against any supported backend without modification.

[rust]: https://www.rust-lang.org/
[wasi-kv]: https://github.com/WebAssembly/wasi-keyvalue

## Prerequisites

- [Wasm Shell (`wash`)][wash]
- [Rust toolchain][rust-install]
- The `wasm32-wasip2` Rust target: `rustup target add wasm32-wasip2`

[wash]: https://wasmcloud.com/docs/installation
[rust-install]: https://www.rust-lang.org/tools/install

## Local development

Use `wash wit fetch` for first-time setup to fetch the `wasi:keyvalue/store` interface.

Use `wash new` to scaffold a new wasmCloud component project:

```shell
wash new https://github.com/wasmCloud/wasmCloud.git --name http-kv-handler --subfolder templates/http-kv-handler
```

```shell
cd http-kv-handler
```

To build this project and run in a hot-reloading development loop, run `wash dev` from this directory:

```shell
wash dev
```

## Endpoints

| Endpoint | Method | Description |
| -------- | ------ | ----------- |
| `/` | POST | Stores a key-value pair from a JSON body `{"key":"...","value":"..."}` |
| `/?key=<key>` | GET | Returns the value stored at `<key>`, or `404` if the key does not exist |

## Send requests to the running component

### Local development (wash dev)

When running with `wash dev`, the HTTP server listens on port 8080 by default:

Store a value

```shell
curl -X POST http://localhost:8000 \
  -H "Content-Type: application/json" \
  -d '{"key":"mykey","value":"myvalue2"}'
```

Retrieve a value

```shell
curl "http://localhost:8000?key=mykey"
```

### Deployment (wadm.yaml)

When deployed via WADM, the included `wadm.yaml` uses the Redis capability provider and connects to `redis://redis:6379` on the `wasmcloud-lattice` Docker network (the `redis` service in `infrastructure/docker-compose.yaml`).

After deployment, use the same port 8080 commands above to test.

## Choosing a backend

The `BACKEND` constant in [src/lib.rs](src/lib.rs) controls which bucket name is passed to `open()`. The host runtime selects the actual storage backend based on [.wash/config.yaml](.wash/config.yaml).

Set `BACKEND` to one of the following values and uncomment the matching section in `.wash/config.yaml`:

| `BACKEND`      | Description                                | Required config key            | Example value                    |
|----------------|--------------------------------------------|--------------------------------|----------------------------------|
| `"in_memory"`  | Ephemeral in-process store (default)       | *(none)*                       |                                  |
| `"filesystem"` | Persists data to a local directory         | `wasi_keyvalue_path`           | `/tmp/keyvalue-store`            |
| `"nats"`       | Uses [NATS][nats] JetStream as the store   | `wasi_keyvalue_nats_url`       | `nats://127.0.0.1:4222`          |
| `"redis"`      | Uses a [Redis][redis] server as the store  | `wasi_keyvalue_redis_url`      | `redis://127.0.0.1:6379`         |

[nats]: https://nats.io/
[redis]: https://redis.io/

### in_memory (default)

No configuration required. Data lives only for the lifetime of the `wash dev` process.

```rust
// src/lib.rs
const BACKEND: &str = "in_memory";
```

```yaml
# .wash/config.yaml — no extra keys needed
dev: {}
```

### filesystem

Data is written to a local directory and survives process restarts.

```rust
// src/lib.rs
const BACKEND: &str = "filesystem";
```

```yaml
# .wash/config.yaml
dev:
  wasi_keyvalue_path: /tmp/keyvalue-store
```

### nats

Requires a running NATS server with JetStream enabled.

```rust
// src/lib.rs
const BACKEND: &str = "nats";
```

```yaml
# .wash/config.yaml
dev:
  wasi_keyvalue_nats_url: nats://127.0.0.1:4222
```

Start a local NATS server with JetStream:

```shell
docker run --name nats -p 4222:4222 nats:latest -js
```

### redis

Requires a running Redis server.

```rust
// src/lib.rs
const BACKEND: &str = "redis";
```

```yaml
# .wash/config.yaml
dev:
  wasi_keyvalue_redis_url: redis://127.0.0.1:6379
```

Start a local Redis server:

```shell
docker run --name redis -p 6379:6379 redis:latest
```

## Build Wasm binary

```shell
wash build
```

## Deployment Commands

### Prerequisites

Check WASH version (must be ≥2.9.0):

```sh
wash --version
```

Check lattice readiness:

```sh
curl -f http://localhost:9090/readyz
```

### Push Component to Registry

Push your compiled WASM to the local HTTP registry (use `--insecure` for non-HTTPS registries):

```sh
wash oci push --insecure localhost:5001/http-kv-handler:0.1.0 ./target/wasm32-wasip2/release/http_kv_handler.wasm
```

### Upload and Deploy

Validate your WASH configuration:

```sh
wash config validate
```

Upload the application manifest to the lattice:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice -v "$PWD/wadm.yaml:/m.yaml:ro" natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.put" "$(cat /m.yaml)" --raw --timeout 10s'
```

Deploy the application:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.http-kv-handler-app" "" --raw --timeout 10s'
```

### Testing & Inspection

List all deployed models in the lattice:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s'
```

Test the application:


Store a value

```shell
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d '{"key":"mykey","value":"myvalue"}'
```

Retrieve a value

```shell
curl "http://localhost:8080?key=mykey"
```

Some Redis Commands:

```
SET mykey "myvalue";

INFO keyspace;

SCAN 0 MATCH * COUNT 1000;
```

### Cleanup

Undeploy the application:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.undeploy.http-kv-handler-app" "" --raw --timeout 10s'
```

Remove the application manifest from the lattice:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.http-kv-handler-app" "" --raw --timeout 10s'
```

## WIT Interfaces

This component uses the following [WIT interfaces](https://component-model.bytecodealliance.org/design/wit.html):

```wit
world http-kv-handler {
  import wasi:keyvalue/store@0.2.0-draft;

  export wasi:http/incoming-handler@0.2.2;
}
```