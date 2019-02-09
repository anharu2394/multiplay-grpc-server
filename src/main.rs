extern crate futures;
extern crate multiplay_grpc_server;
extern crate serde_derive;
#[macro_use(bson, doc)]
extern crate mongodb;

mod protos;

use futures::Future;
use futures::sync::oneshot;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::io::Read;

use grpcio::*;
use protos::multiplay::*;
use protos::multiplay_grpc::{Multiplay, User};

use diesel::prelude::*;
use diesel::pg::PgConnection;

use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use multiplay_grpc_server::schema::users;
use multiplay_grpc_server::schema::users::dsl::*;
use multiplay_grpc_server::{establish_connection};
use multiplay_grpc_server::models::*;

#[derive(Clone)]
struct MultiplayService;

#[derive(Clone)]
struct UserService;

impl Multiplay for MultiplayService {
    fn get_users(&mut self,
        ctx: RpcContext,
        req: GetUsersRequest,
        resp: ServerStreamingSink<GetUsersResponse>
        ) {
        println!("{}",req.get_room_id());
    }
    fn set_position(&mut self,
        ctx: RpcContext,
        req: RequestStream<SetPositionRequest>,
        resp: ClientStreamingSink<SetPositionResponse>
        ) {
    }
}

impl User for UserService {
    fn create(&mut self, ctx: RpcContext, req: CreateUserRequest, sink: UnarySink<CreateUserResponse>) {
        let conn = establish_connection();
        let user_name = req.get_name();
        println!("{}", &user_name);
        let new_user = NewUser { name: user_name.to_string() };
        let result_id = diesel::insert_into(users).values(&new_user).returning(id).get_result::<i32>(&conn).unwrap();
        let mut resp = CreateUserResponse::new();
        resp.set_id(result_id as u32);
        let f = sink
            .success(resp)
            .map_err(move |e| println!("failed to reply {:?}: {:?}", req, e));
        ctx.spawn(f)
    }
}

fn main() {
    let client = Client::connect("localhost", 27017)
        .expect("Failed to initialize standalone client.");

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
