# Hello World Prebuild

## Configuration Files

### `wadm.yaml`
This is a wasmCloud Application Deployment Model (WADM) manifest that describes your application topology:
- **HTTP Component**: A prebuilt HTTP hello-world component (`ghcr.io/wasmcloud/components/http-hello-world-rust:0.1.0`) deployed with 2 instances across the lattice
- **HTTP Server Capability**: Provides the HTTP server implementation
- **Link**: Connects the component to the capability, binding the HTTP server to listen on `0.0.0.0:8080`

This declarative manifest ensures consistent deployment across your wasmCloud lattice.

### `.wash/config.yaml`
Configuration file for the WASH CLI (wasmCloud Shell):
- **version**: Specifies the WASH CLI version (2.9.0) — determines compatibility with your lattice
- **dev**: Development settings (e.g., `allow_insecure_registries: false` for secure registry operations)

## Quick Start Commands

### Prerequisites

Check WASH version (must be ≥2.9.0)

```sh
wash --version
```

Check lattice readiness

```sh
curl -f http://localhost:9090/readyz
```

### Deployment

Validate your WASH configuration

```sh
wash config validate
```

Upload the application manifest to the lattice

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice -v "$PWD/wadm.yaml:/m.yaml:ro" natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.put" "$(cat /m.yaml)" --raw --timeout 10s'
```

Deploy the application

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.hello-world-app" "" --raw --timeout 10s'
```

### Testing & Inspection

List all deployed models in the lattice

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s'
```

Test the application

```sh
curl localhost:8080
```

### Cleanup

Undeploy the application

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.undeploy.hello-world-app" "" --raw --timeout 10s'
```

Remove the application manifest from the lattice

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.hello-world-app" "" --raw --timeout 10s'
```
