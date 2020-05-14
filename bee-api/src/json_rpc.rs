use jsonrpc_http_server::*;
use jsonrpc_http_server::jsonrpc_core::*;

pub fn rpc_server() -> Server {

    let mut io = IoHandler::default();
    io.add_method("hello", |_| {
        Ok(Value::String("Hello World".into()))
    });

    ServerBuilder::new(io)
        .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Null]))
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .expect("Unable to start RPC server")

}