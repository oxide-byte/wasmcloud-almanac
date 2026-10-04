deploy:
  docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.del.http-kv-handler-app" "" --raw --timeout 10s'
  wash build
  wash oci push --insecure localhost:5001/http-kv-handler:0.1.0 ./target/wasm32-wasip2/release/http_kv_handler.wasm
  wash config validate
  docker run --rm --network wasmcloud-almanac_wasmcloud-lattice -v "$PWD/wadm.yaml:/m.yaml:ro" natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.put" "$(cat /m.yaml)" --raw --timeout 10s'
  docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'nats -s nats://nats:4222 req "wadm.api.default.model.deploy.http-kv-handler-app" "" --raw --timeout 10s'
  docker run --rm --network wasmcloud-almanac_wasmcloud-lattice natsio/nats-box:latest sh -c 'for i in $(seq 1 30); do out=$(nats -s nats://nats:4222 req "wadm.api.default.model.list" "" --raw --timeout 10s); echo "$out" | grep -q "\"status\":\"deployed\"" && echo "$out" && exit 0; echo "waiting for deployment ($i/30)..."; sleep 2; done; echo "$out"; echo "timed out waiting for deployed status"; exit 1'
  curl --retry 10 --retry-delay 1 --retry-all-errors -X POST http://localhost:8080 -H "Content-Type: application/json" -d '{"key":"mykey","value":"myvalue"}'
  curl --retry 10 --retry-delay 1 --retry-all-errors "http://localhost:8080?key=mykey"
