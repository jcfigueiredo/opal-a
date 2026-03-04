use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opal_runtime::Value;

/// A registered route in the HTTP router
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler_closure_id: usize,
}

/// A simple HTTP router that stores routes
pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }
}

/// Register the `http` plugin and return its native functions.
///
/// Functions provided:
/// - `create_app()` -> Integer (app ID)
/// - `add_route(app_id, method, path, handler_closure)` -> Null
/// - `get_routes(app_id)` -> List of [method, path, handler_id] lists
/// - `serve(app_id, port)` -> placeholder (intercepted by interpreter)
pub fn register_http_plugin() -> HashMap<String, super::NativeFunction> {
    let routers: Arc<Mutex<Vec<Router>>> = Arc::new(Mutex::new(Vec::new()));

    let mut fns = HashMap::new();

    // create_app() — creates a new Router, returns its integer ID
    let routers_create = Arc::clone(&routers);
    fns.insert(
        "create_app".to_string(),
        Box::new(
            move |_args: &[Value], _w: &mut dyn std::io::Write| -> Result<Value, String> {
                let mut routers = routers_create.lock().unwrap();
                let id = routers.len();
                routers.push(Router::new());
                Ok(Value::Integer(id as i64))
            },
        ) as super::NativeFunction,
    );

    // add_route(app_id, method, path, handler) — stores a route
    let routers_add = Arc::clone(&routers);
    fns.insert(
        "add_route".to_string(),
        Box::new(
            move |args: &[Value], _w: &mut dyn std::io::Write| -> Result<Value, String> {
                if args.len() < 4 {
                    return Err("add_route requires 4 arguments: app, method, path, handler".into());
                }
                let app_id = match &args[0] {
                    Value::Integer(id) => *id as usize,
                    _ => return Err("expected integer app id".into()),
                };
                let method = match &args[1] {
                    Value::String(s) => s.clone(),
                    _ => return Err("expected method string".into()),
                };
                let path = match &args[2] {
                    Value::String(s) => s.clone(),
                    _ => return Err("expected path string".into()),
                };
                let handler_id = match &args[3] {
                    Value::Closure(id) => id.0,
                    _ => return Err("expected closure handler".into()),
                };

                let mut routers = routers_add.lock().unwrap();
                if let Some(router) = routers.get_mut(app_id) {
                    router.routes.push(Route {
                        method,
                        path,
                        handler_closure_id: handler_id,
                    });
                    Ok(Value::Null)
                } else {
                    Err(format!("invalid app id: {}", app_id))
                }
            },
        ) as super::NativeFunction,
    );

    // get_routes(app_id) — returns routes as [[method, path, handler_id], ...]
    let routers_get = Arc::clone(&routers);
    fns.insert(
        "get_routes".to_string(),
        Box::new(
            move |args: &[Value], _w: &mut dyn std::io::Write| -> Result<Value, String> {
                if args.is_empty() {
                    return Err("get_routes requires 1 argument: app_id".into());
                }
                let app_id = match &args[0] {
                    Value::Integer(id) => *id as usize,
                    _ => return Err("expected integer app id".into()),
                };
                let routers = routers_get.lock().unwrap();
                if let Some(router) = routers.get(app_id) {
                    let route_values: Vec<Value> = router
                        .routes
                        .iter()
                        .map(|r| {
                            Value::List(vec![
                                Value::String(r.method.clone()),
                                Value::String(r.path.clone()),
                                Value::Integer(r.handler_closure_id as i64),
                            ])
                        })
                        .collect();
                    Ok(Value::List(route_values))
                } else {
                    Err(format!("invalid app id: {}", app_id))
                }
            },
        ) as super::NativeFunction,
    );

    // serve(app_id, port) — placeholder; the real serve logic is intercepted by the interpreter
    fns.insert(
        "serve".to_string(),
        Box::new(
            |_args: &[Value], _w: &mut dyn std::io::Write| -> Result<Value, String> {
                // This should never be called directly — the interpreter intercepts http:serve
                Err("serve must be intercepted by the interpreter".into())
            },
        ) as super::NativeFunction,
    );

    fns
}
