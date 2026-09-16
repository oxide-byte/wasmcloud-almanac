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

```sh
wasm-tools component wit ./target/wasm32-wasip2/release/hello_world.wasm
```

produces the following wit definition:

```
package root:component;

world root {
  import wasi:io/poll@0.2.9;
  import wasi:clocks/monotonic-clock@0.2.9;
  import wasi:io/error@0.2.9;
  import wasi:io/streams@0.2.9;
  import wasi:cli/stdout@0.2.9;
  import wasi:cli/stderr@0.2.9;
  import wasi:cli/stdin@0.2.9;
  import wasi:http/types@0.2.9;
  import wasi:cli/environment@0.2.9;
  import wasi:cli/exit@0.2.9;
  import wasi:cli/terminal-input@0.2.9;
  import wasi:cli/terminal-output@0.2.9;
  import wasi:cli/terminal-stdin@0.2.9;
  import wasi:cli/terminal-stdout@0.2.9;
  import wasi:cli/terminal-stderr@0.2.9;
  import wasi:random/insecure-seed@0.2.9;

  export wasi:http/incoming-handler@0.2.9;
}
package wasi:io@0.2.9 {
  interface poll {
    resource pollable {
      ready: func() -> bool;
      block: func();
    }

    poll: func(in: list<borrow<pollable>>) -> list<u32>;
  }
  interface error {
    resource error {
      to-debug-string: func() -> string;
    }
  }
  interface streams {
    use poll.{pollable};
    use error.{error};

    resource input-stream {
      subscribe: func() -> pollable;
    }

    resource output-stream {
      check-write: func() -> result<u64, stream-error>;
      write: func(contents: list<u8>) -> result<_, stream-error>;
      blocking-flush: func() -> result<_, stream-error>;
      subscribe: func() -> pollable;
      splice: func(src: borrow<input-stream>, len: u64) -> result<u64, stream-error>;
    }

    variant stream-error {
      last-operation-failed(error),
      closed,
    }
  }
}


package wasi:clocks@0.2.9 {
  interface monotonic-clock {
    use wasi:io/poll@0.2.9.{pollable};

    type duration = u64;

    subscribe-duration: func(when: duration) -> pollable;
  }
}


package wasi:cli@0.2.9 {
  interface stdout {
    use wasi:io/streams@0.2.9.{output-stream};

    get-stdout: func() -> output-stream;
  }
  interface stderr {
    use wasi:io/streams@0.2.9.{output-stream};

    get-stderr: func() -> output-stream;
  }
  interface stdin {
    use wasi:io/streams@0.2.9.{input-stream};

    get-stdin: func() -> input-stream;
  }
  interface environment {
    get-environment: func() -> list<tuple<string, string>>;
  }
  interface exit {
    exit: func(status: result);
  }
  interface terminal-input {
    resource terminal-input;
  }
  interface terminal-output {
    resource terminal-output;
  }
  interface terminal-stdin {
    use terminal-input.{terminal-input};

    get-terminal-stdin: func() -> option<terminal-input>;
  }
  interface terminal-stdout {
    use terminal-output.{terminal-output};

    get-terminal-stdout: func() -> option<terminal-output>;
  }
  interface terminal-stderr {
    use terminal-output.{terminal-output};

    get-terminal-stderr: func() -> option<terminal-output>;
  }
}


package wasi:http@0.2.9 {
  interface types {
    use wasi:io/streams@0.2.9.{input-stream};
    use wasi:io/poll@0.2.9.{pollable};
    use wasi:io/streams@0.2.9.{output-stream};

    resource outgoing-body {
      write: func() -> result<output-stream>;
      finish: static func(this: outgoing-body, trailers: option<trailers>) -> result<_, error-code>;
    }

    resource fields {
      constructor();
      append: func(name: field-name, value: field-value) -> result<_, header-error>;
      entries: func() -> list<tuple<field-name, field-value>>;
    }

    resource incoming-body {
      %stream: func() -> result<input-stream>;
      finish: static func(this: incoming-body) -> future-trailers;
    }

    resource future-trailers {
      subscribe: func() -> pollable;
      get: func() -> option<result<result<option<trailers>, error-code>>>;
    }

    resource incoming-request {
      method: func() -> method;
      path-with-query: func() -> option<string>;
      scheme: func() -> option<scheme>;
      authority: func() -> option<string>;
      headers: func() -> headers;
      consume: func() -> result<incoming-body>;
    }

    resource response-outparam {
      set: static func(param: response-outparam, response: result<outgoing-response, error-code>);
    }

    resource outgoing-response {
      constructor(headers: headers);
      set-status-code: func(status-code: status-code) -> result;
      body: func() -> result<outgoing-body>;
    }

    record DNS-error-payload {
      rcode: option<string>,
      info-code: option<u16>,
    }

    record TLS-alert-received-payload {
      alert-id: option<u8>,
      alert-message: option<string>,
    }

    record field-size-payload {
      field-name: option<string>,
      field-size: option<u32>,
    }

    variant error-code {
      DNS-timeout,
      DNS-error(DNS-error-payload),
      destination-not-found,
      destination-unavailable,
      destination-IP-prohibited,
      destination-IP-unroutable,
      connection-refused,
      connection-terminated,
      connection-timeout,
      connection-read-timeout,
      connection-write-timeout,
      connection-limit-reached,
      TLS-protocol-error,
      TLS-certificate-error,
      TLS-alert-received(TLS-alert-received-payload),
      HTTP-request-denied,
      HTTP-request-length-required,
      HTTP-request-body-size(option<u64>),
      HTTP-request-method-invalid,
      HTTP-request-URI-invalid,
      HTTP-request-URI-too-long,
      HTTP-request-header-section-size(option<u32>),
      HTTP-request-header-size(option<field-size-payload>),
      HTTP-request-trailer-section-size(option<u32>),
      HTTP-request-trailer-size(field-size-payload),
      HTTP-response-incomplete,
      HTTP-response-header-section-size(option<u32>),
      HTTP-response-header-size(field-size-payload),
      HTTP-response-body-size(option<u64>),
      HTTP-response-trailer-section-size(option<u32>),
      HTTP-response-trailer-size(field-size-payload),
      HTTP-response-transfer-coding(option<string>),
      HTTP-response-content-coding(option<string>),
      HTTP-response-timeout,
      HTTP-upgrade-failed,
      HTTP-protocol-error,
      loop-detected,
      configuration-error,
      internal-error(option<string>),
    }

    type trailers = fields;

    type headers = fields;

    type status-code = u16;

    type field-key = string;

    type field-name = field-key;

    type field-value = list<u8>;

    variant header-error {
      invalid-syntax,
      forbidden,
      immutable,
    }

    variant method {
      get,
      head,
      post,
      put,
      delete,
      connect,
      options,
      trace,
      patch,
      other(string),
    }

    variant scheme {
      HTTP,
      HTTPS,
      other(string),
    }
  }
  interface incoming-handler {
    use types.{incoming-request, response-outparam};

    handle: func(request: incoming-request, response-out: response-outparam);
  }
}


package wasi:random@0.2.9 {
  interface insecure-seed {
    insecure-seed: func() -> tuple<u64, u64>;
  }
}
```