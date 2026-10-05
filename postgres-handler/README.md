# HTTP + Postgres Handler in Rust

A WebAssembly component built with [Rust][rust] that serves a read-only list of users over HTTP, queried from [Postgres][postgres] through the `wasmcloud:postgres/query` interface.

The component only runs `SELECT id, name, email FROM users ORDER BY id`. There are no writes. Connection details are not in the component: the host or capability provider owns them.

[rust]: https://www.rust-lang.org/
[postgres]: https://www.postgresql.org/

## Prerequisites

- [Wasm Shell (`wash`)][wash]
- [Rust toolchain][rust-install]
- The `wasm32-wasip2` Rust target: `rustup target add wasm32-wasip2`
- The Postgres service from `infrastructure/docker-compose.yaml` (database `wasmdb`, seeded with `users` by `infrastructure/postgres/init.sql`, published on `localhost:5432`)
- [`just`][just] (only for the one-step deploy)

Start the cluster, including Postgres, from `infrastructure/`:

```shell
docker compose up -d --build
```

[just]: https://github.com/casey/just
[wash]: https://wasmcloud.com/docs/installation
[rust-install]: https://www.rust-lang.org/tools/install

## Endpoints

| Endpoint | Method | Description |
| -------- | ------ | ----------- |
| `/` | GET | Returns all users as a JSON array |

Other methods return `405`. A failed query returns `500`.

Example response:

```json
[
  {"id":1,"name":"Alice","email":"alice@example.com"},
  {"id":2,"name":"Bob","email":"bob@example.com"},
  {"id":3,"name":"Charlie","email":"charlie@example.com"}
]
```

## Build Wasm binary

```shell
wash build
```

The output is `target/wasm32-wasip2/release/postgres_handler.wasm`.

## Local development

`.wash/config.yaml` configures `wash dev` to connect to the Compose Postgres at `postgres://postgres:postgres@127.0.0.1:5432/wasmdb` and to provide `wasmcloud:postgres/query` to the component:

```shell
wash dev
```

## Deployment (wadm.yaml)

The included `wadm.yaml` deploys the component with:

- the HTTP server provider on port 8080
- a Postgres provider (`ghcr.io/wasmcloud/sqldb-postgres:canary`), linked to the component for `wasmcloud:postgres/query`

The link config points at the `postgres` Compose service on the `wasmcloud-lattice` network (`postgres:5432`, database `wasmdb`, user/password `postgres`).

> **Unverified:** the provider image tag and the `POSTGRES_*` config key names in `wadm.yaml` have not been checked against a running provider. Confirm them if the provider fails to connect.

### Prerequisites

Check WASH version (must be ≥2.9.0):

```sh
wash --version
```

Check lattice readiness:

```sh
curl -f http://localhost:9090/readyz
```

### One-step deploy

The [`.justfile`](.justfile) runs the whole flow (undeploy, build, push, upload manifest, deploy, wait, smoke test):

```shell
just deploy
```

It polls wadm until the application status is `deployed` before sending the test request, because the providers take several seconds to start. Requests sent earlier fail with `curl: (52) Empty reply from server`.

### Manual steps

Push the compiled component to the local registry:

```sh
wash oci push --insecure localhost:5001/postgres-handler:0.1.0 ./target/wasm32-wasip2/release/postgres_handler.wasm
```

Upload the manifest:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice -v "$PWD/wadm.yaml:/m.yaml:ro" natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.put" "$(cat /m.yaml)" --raw --timeout 10s'
```

Deploy:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.postgres-handler-app" "" --raw --timeout 10s'
```

List deployed models:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s'
```

### Test

```shell
curl http://localhost:8080
```

### Troubleshooting

| Symptom | Cause |
| ------- | ----- |
| `curl: (52) Empty reply from server` | Providers are still starting (`reconciling`). Wait for status `deployed`. |
| `500` with `incomplete results` or `not connected` | The component's calls have no serving provider. Check the host logs: `docker logs wasmcloud-almanac-wasmcloud-1`. |
| `500` with `Query failed: ...` | The provider is up but the query failed. Check the connection config and that `users` exists in `wasmdb`. |

Check the database directly:

```sh
docker exec postgres psql -U postgres -d wasmdb -c 'SELECT * FROM users;'
```

### Cleanup

Undeploy:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.undeploy.postgres-handler-app" "" --raw --timeout 10s'
```

Remove the manifest:

```sh
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.postgres-handler-app" "" --raw --timeout 10s'
```

## WIT Interfaces

The component's world ([wit/world.wit](wit/world.wit)):

```wit
world postgres-handler {
  import wasmcloud:postgres/query@0.1.1-draft;
}
```

The `wasi:http/incoming-handler` export is added by the `wstd::http_server` macro. The `wasmcloud:postgres` WIT lives in `wit/deps/wasmcloud-postgres-0.1.1-draft`.
