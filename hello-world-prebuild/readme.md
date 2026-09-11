# Hello World Prebuild

## Commands

check readiness

```sh
curl -f http://localhost:9090/readyz
```

Check WASH version (>=2.9.0)

```sh
wash --version
```

check host

```
wash host
```

upload wadm hello-world

```
wash host
```

validate config

```sh
wash config validate
```

validate module

```sh
curl localhost:8080 
```

Upload wadm hello-world

```
 docker run --rm --network wasmcloud-almanac_wasmcloud-lattice -v "$PWD/wadm.yaml:/m.yaml:ro" natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.put" "$(cat /m.yaml)" --raw --timeout 10s'
```

Deploy

```
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.hello-world-app" "" --raw --timeout 10s'
```

List models

```
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s'
```

```sh
curl localhost:8080 
```

```
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.undeploy.hello-world-app" "" --raw --timeout 10s'
```

```
docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.hello-world-app" "" --raw --timeout 10s'
```
