extern crate futures;
extern crate multiplay_grpc_server;
extern crate serde_derive;
extern crate mongodb;

mod protos;

use futures::*;
use futures::Stream;
use futures::sync::oneshot;
use std::env;
use std::sync::{Arc};
use std::{io, thread};
use std::io::Read;

use grpcio::*;
use protos::multiplay::*;
use protos::multiplay_grpc::{Multiplay, User};

use mongodb::{Bson, bson, doc};
use mongodb::oid::ObjectId;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;


#[derive(Clone)]
struct MultiplayService {
    client: Client
}

#[derive(Clone)]
struct UserService {
    client: Client
}
impl Multiplay for MultiplayService {
    fn get_users(&mut self,
        ctx: RpcContext,
        req: GetUsersRequest,
        resp: ServerStreamingSink<GetUsersResponse>
        ) {
        println!("{}",req.get_room_id());
        let db = &self.client;
    }
    fn set_position(&mut self,
        ctx: RpcContext,
        req: RequestStream<SetPositionRequest>,
        resp: ClientStreamingSink<SetPositionResponse>
        ) {
        let coll = self.client.db("multiplay-grpc").collection("users");
        println!("get!!request");
        let f = req.map(move |position| {
            println!("Receive: {:?}", position);
            let id = position.get_id().to_string();
            let filter = doc!{"_id": ObjectId::with_string(&id).unwrap()};
            let new_position = doc!{
                "$set": {
                    "x": position.get_x(),
                    "y": position.get_y(),
                },
            };
            let coll_result = coll.find_one_and_update(filter.clone(), new_position, None)
                .expect("Faild to get player");
            let player = coll_result.expect("result is None");
            println!("player : {}",player);
            /*
            match coll.update_one(filter.clone(), new_position, None) {
                Ok(r) => {
                    println!("{} was matched {} was modified",r.matched_count,r.modified_count);
                match r.write_exception {
                    Some(exce) => println!("{}",exce.message),
                    None => println!("no exception"),
                }
                },
                Err(e) => panic!("{}",e),
            }
            */
            id
        })
        .fold(String::new(),|init,id| {
            println!("init :{}",init);
            println!("id: {}",id);
            Ok(format!("{}",id)) as Result<String>
        })
        .and_then(move |id| {
            let mut rep = SetPositionResponse::new();
            rep.set_id(id);
            rep.set_status("ok".to_string());
            resp.success(rep)
        })
        .map_err(|e| println!("failed to record route: {:?}", e));
        ctx.spawn(f)
    }
}

impl User for UserService {
    fn create(&mut self, ctx: RpcContext, req: CreateUserRequest, sink: UnarySink<CreateUserResponse>) {
        let coll = self.client.db("multiplay-grpc").collection("users");
        let user_name = req.get_name();
        println!("{}", &user_name);
        let new_user = doc! {
            "name": user_name,
            "x": 0,
            "y": 0,
        };
        let result_bson = coll.insert_one(new_user.clone(), None)
            .expect("Failed to insert doc.").inserted_id.expect("Failed to get inserted id");
        let result_id = result_bson.as_object_id().unwrap().to_hex();
        println!("{}",&result_id); 
        let mut resp = CreateUserResponse::new();
        resp.set_id(result_id);
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
    let userService = protos::multiplay_grpc::create_user(UserService { client: client.clone()});
    let multiplayService = protos::multiplay_grpc::create_multiplay(MultiplayService { client: client.clone() });
    let host = env::var("RUST_GRPC_HOST").unwrap_or("127.0.0.1".to_string()); 
    let port = env::var("RUST_GRPC_PORT").unwrap_or("57601".to_string());
    let mut server = ServerBuilder::new(env)
        .register_service(userService)
        .register_service(multiplayService)
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
