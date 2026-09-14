# HTTP Hello World Template in Rust

A minimal WebAssembly component built with [Rust][rust] that responds to HTTP requests using the [`wstd`][wstd] async standard library and its `#[http_server]` proc macro.

[rust]: https://www.rust-lang.org/
[wstd]: https://github.com/bytecodealliance/wstd

## Prerequisites

- [Wasm Shell (`wash`)][wash]
- [Rust toolchain][rust-install]
- The `wasm32-wasip2` Rust target: `rustup target add wasm32-wasip2`

[wash]: https://wasmcloud.com/docs/installation
[rust-install]: https://www.rust-lang.org/tools/install

## Local development

Use `wash new` to scaffold a new wasmCloud component project:

```sh
wash new https://github.com/wasmCloud/wasmCloud.git --name hello-world-template --subfolder templates/http-hello-world
```

Navigate to the project directory:

```sh
cd hello-world-template
```

To build this project and run in a hot-reloading development loop:

```sh
wash dev
```

### Send a request to the running component

Once `wash dev` is serving your component, send a request:

```sh
curl localhost:8000
```

Response:

```text
Hello from wasmCloud!
```

## Build Wasm binary

```sh
wash build
```

## Configuration Files

### `wadm.yaml`
This is a wasmCloud Application Deployment Model (WADM) manifest that describes your application topology:
- **HTTP Component**: Your locally built HTTP hello-world component deployed with 2 instances across the lattice
- **HTTP Server Capability**: Provides the HTTP server implementation
- **Link**: Connects the component to the capability, binding the HTTP server to listen on `0.0.0.0:8080`

This declarative manifest ensures consistent deployment across your wasmCloud lattice.

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
wash oci push --insecure localhost:5001/hello-world:0.1.0 ./target/wasm32-wasip2/release/hello_world.wasm
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
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.hello-world-template-app" "" --raw --timeout 10s'
```

### Testing & Inspection

List all deployed models in the lattice:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s'
```

Test the application:

```sh
curl localhost:8080
```

### Cleanup

Undeploy the application:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.undeploy.hello-world-template-app" "" --raw --timeout 10s'
```

Remove the application manifest from the lattice:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.hello-world-template-app" "" --raw --timeout 10s'
```

## WIT Interfaces

This component exports `wasi:http/incoming-handler` via `wstd`'s `#[http_server]` proc macro, so the [WIT world](https://component-model.bytecodealliance.org/design/wit.html) itself is empty:

```wit
world hello {}
```
