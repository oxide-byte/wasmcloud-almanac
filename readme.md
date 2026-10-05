# WasmCloud Almanac

## Introduction

Part of the documentation and project is based on the official documentation of WasmCloud.
See: https://wasmcloud.com/docs/. A major problem currently is the distinction between V1 and V2.
The breaking changes are difficult for the AI to separate and getting working samples is challenging.

## Target

Due this challenges, I worked over the samples and build this project. Later it will be extended to use cases not covered by the official documentation but in an personal interest.

The documentation is build with mdBook https://rust-lang.github.io/mdBook/ 

The main documentation will be under GitHub Pages: https://...

Main goal is having all running on a local Docker-Compose Cluster, that it could be replicated as close as possible to the production environment.

## Projects

| Directory | Description |
| --------- | ----------- |
| [infrastructure](infrastructure) | Docker Compose cluster (NATS, wasmCloud host, wadm, registry, observability, Postgres seeded with a `users` table) |
| [hello-world-prebuild](hello-world-prebuild) | Deploys a prebuilt hello-world component via wadm |
| [hello-world-template](hello-world-template) | Minimal HTTP hello-world component in Rust |
| [http-kv-handler](http-kv-handler) | HTTP component that stores and retrieves key-value pairs via `wasi:keyvalue` (Redis) |
| [postgres-handler](postgres-handler) | HTTP component that returns the `users` table from Postgres (read-only `GET`) via `wasmcloud:postgres/query` |
| [wasm-dashboard](wasm-dashboard) | Dioxus Fullstack app to manage wasmCloud applications |
| [mdbook](mdbook) | Source of the documentation |

The `infrastructure` Postgres service (database `wasmdb`, user/password `postgres`, port `5432`) is built from `infrastructure/postgres` and seeded by `init.sql`. It is used by `postgres-handler`.

## Bonus

The project contains an Dioxus Fullstack application. This application can be used to manage your wasmCloud applications in the cluster.

## AI

The project has been assisted by AI tools to generate the documentation and code. But as mentioned, AI has a lot of knowledge gaps on the wasmCloud ecosystem. I enjoyed to clash with these limitations as on the one it shows an other truth about AI and on the other it helped me to learn more about wasmCloud.

## Disclaimer

As WebAssembly and WASI moves and changes fast, there are some challenges to keep up with the latest versions.