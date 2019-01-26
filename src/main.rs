extern crate futures;

mod protos;

use futures::Future;
use futures::sync::oneshot;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::io::Read;

use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};
use protos::multiplay::*;
use protos::multiplay_grpc::{Multiplay, User};

#[derive(Clone)]
struct MultiplayService;

#[derive(Clone)]
struct UserService;

impl Multiplay for MultiplayService {
}

impl User for UserService {
    fn create(&mut self, ctx: RpcContext, req: CreateUserRequest, sink: UnarySink<CreateUserResponse>) {
        let name = req.get_name();
        println!("{}", name);
        let mut resp = CreateUserResponse::new();
        resp.set_id(1);
        let f = sink
            .success(resp)
            .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
        ctx.spawn(f)
    }
}

fn main() {
    println!("hello world");
    let env = Arc::new(Environment::new(1));
    let service = protos::multiplay_grpc::create_user(UserService);
    let host = env::var("RUST_GRPC_HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("RUST_GRPC_PORT").unwrap_or("57601".to_string());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind(host, port.parse().unwrap())
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = rx.wait();
    let _ = server.shutdown().wait();
}
