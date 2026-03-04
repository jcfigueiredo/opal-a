# Slice 8 Design: Web Server Capstone with FFI

## Goal

Build a working web server in Opal that proves the language works end-to-end — from macros to FFI to HTTP request handling. Introduces a plugin-based FFI system (`extern "name"`) and uses it to build an HTTP server with `@get`/`@post` macro routing.

## Architecture

Three layers:

```
Opal user code          @get "/" do "Hello!" end
       ↓ (macro expansion)
OpalWeb module          app.add_route("GET", "/", handler)
       ↓ (method calls)
FFI plugin "http"       Rust TcpListener + HTTP/1.1 parser
```

### FFI Plugin System

`extern "name"` blocks declare foreign functions. The interpreter resolves the name against a `PluginRegistry` — a `HashMap<String, HashMap<String, NativeFunction>>` where `NativeFunction = fn(&[Value], &mut dyn Write) -> Result<Value, String>`.

Plugins are registered at interpreter startup in Rust code. No dynamic library loading (Phase 1) — plugins are compiled into the binary.

```opal
extern "http"
  def create_app() -> App
  def add_route(app, method: String, path: String, handler: Fn) -> Null
  def run(app, port: Int) -> Null
end
```

Type annotations are parsed but not enforced (consistent with Phase 1).

### HTTP Plugin (Rust)

The `http` plugin uses `std::net::TcpListener` — zero external dependencies:

- `http_create_app()` → returns a `Value::NativeObject(id)` wrapping a router struct
- `http_add_route(app, method, path, handler)` → stores `(method, path_pattern, closure_id)` in the router
- `http_run(app, port)` → binds TCP, accept loop, for each connection:
  1. Read request line: `GET /path HTTP/1.1`
  2. Parse method + path
  3. Match against registered routes (simple prefix match, `:param` extraction)
  4. Call the Opal handler closure with a params dict
  5. Write HTTP response: `HTTP/1.1 200 OK\r\n\r\n{body}`

Single-threaded, synchronous. No HTTPS, no HTTP/2, no keep-alive. Sufficient for demo.

### OpalWeb Module (Opal)

Written in Opal, provides the user-facing API:

```opal
module OpalWeb
  class App
    needs name: String

    def add_route(method, path, handler)
      # delegates to FFI
    end

    def run!(port)
      # delegates to FFI
    end
  end
end
```

### @get / @post Macros

```opal
macro get(path, handler)
  ast
    app.add_route("GET", $path, $handler)
  end
end
```

### Target Program (simplified)

```opal
import OpalWeb

app = OpalWeb.App.new("my app")

@get "/" do
  "Hello, world!"
end

@get "/greet/:name" do |params|
  f"Hello, {params.name}!"
end

app.run!(8080)
```

## Key Design Decisions

1. **Plugin registry, not dlopen** — keeps Phase 1 simple and safe. Real FFI with shared libraries deferred to Phase 2.
2. **std::net, not hyper** — zero dependencies. HTTP parsing is ~150 lines of Rust. Async/hyper deferred to Phase 2 when tokio is added for actors.
3. **NativeObject value type** — new `Value::NativeObject(id)` for opaque Rust-side state (the router). Interpreter doesn't inspect it, just passes it through to FFI calls.
4. **Handler execution challenge** — the TCP accept loop runs in Rust, but handlers are Opal closures. The plugin needs a callback mechanism to invoke the interpreter. Solution: the `http_run` function receives the interpreter reference (via the writer/callback pattern already used for print).

## Scope

**In scope:** extern parsing, plugin registry, http plugin, OpalWeb module, @get macro, basic routing, path params, the target program working.

**Out of scope:** JSON, POST body parsing, middleware, HTTPS, HTTP/2, async, Database integration. These are Phase 2.
