mod protos;

use futures::*;
use futures::Stream;
use futures::sync::oneshot;
use std::env;
use std::iter;
use std::sync::{Arc};
use std::{io, thread};
use std::io::Read;

use protobuf::RepeatedField;
use grpcio::*;
use protos::multiplay::*;
use protos::multiplay_grpc::{Multiplay, User};

use mongodb::{bson, doc};
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
        let db = self.client.db("multiplay-grpc").clone();
        let users = iter::repeat(())
            .map(move |()| {
                let coll = db.collection("users");
                let mut reply = GetUsersResponse::new();
                let result_users = coll.find(None, None)
                    .expect("Failed to get users");
                let users_vec: Vec<UserPosition> = result_users
                    .map(|user| {
                        let mut user_position = UserPosition::new();
                        let doc = user.unwrap();
                        user_position.set_id(doc.get_object_id("_id").unwrap().to_hex());
                        user_position.set_x(doc.get_f64("x").unwrap());
                        user_position.set_y(doc.get_f64("y").unwrap());
                        user_position.set_y(doc.get_f64("z").unwrap());
                        user_position
                    })
                    .collect();
                reply.set_users(RepeatedField::from_vec(users_vec));
                (reply, WriteFlags::default())
            });
        let f = resp
            .send_all(stream::iter_ok::<_, Error>(users))
            .map(|_| {})
            .map_err(|e| println!("failed to handle listfeatures request: {:?}", e));
        ctx.spawn(f)
    }
    fn set_position(&mut self,
        ctx: RpcContext,
        req: RequestStream<SetPositionRequest>,
        resp: ClientStreamingSink<SetPositionResponse>
        ) {
        let db = self.client.db("multiplay-grpc").clone();
        println!("get!!request");
        let f = req.map(move |position| {
            let coll = db.collection("users");
            println!("Receive: {:?}", position);
            let id = position.get_id().to_string();
            let filter = doc!{"_id": ObjectId::with_string(&id).unwrap()};
            let new_position = doc!{
                "$set": {
                    "x": position.get_x(),
                    "y": position.get_y(),
                    "z": position.get_z(),
                },
            };
            let coll_result = coll.find_one_and_update(filter.clone(), new_position, None)
                .expect("Faild to get player");
            let player = coll_result.expect("result is None");
            println!("player : {}",player);
            id
        })
        .collect()
        .and_then(|ids| {
            let id = ids.first().unwrap();
            let mut rep = SetPositionResponse::new();
            rep.set_id(id.clone());
            rep.set_status("ok".to_string());
            resp.success(rep)
        })
        .map_err(|e| println!("failed to record route: {:?}", e));
        ctx.spawn(f)
    }
    fn connect_position(&mut self,
        ctx: RpcContext,
        req: RequestStream<ConnectPositionRequest>,
        resp: DuplexSink<ConnectPositionResponse>
        ) {
        let db = self.client.db("multiplay-grpc").clone();
        let to_send = req
            .map(move |position| {
                println!("Receive: {:?}", position);
                let coll = db.collection("users");
                let id = position.get_id().to_string();
                let filter = doc!{"_id": ObjectId::with_string(&id).unwrap()};
                let new_position = doc!{
                    "$set": {
                        "x": position.get_x(),
                        "y": position.get_y(),
                        "z": position.get_z(),
                    },
                };
                let coll_result = coll.find_one_and_update(filter.clone(), new_position, None)
                    .expect("Faild to get player");
                let player = coll_result.expect("result is None");
                println!("player : {}",player);
                
                let result_users = coll.find(None, None)
                    .expect("Failed to get users");
                let users = result_users
                    .map(move |user| {
                        let mut user_position = UserPosition::new();
                        let doc = user.unwrap();
                        user_position.set_id(doc.get_object_id("_id").unwrap().to_hex());
                        user_position.set_x(doc.get_f64("x").unwrap());
                        user_position.set_y(doc.get_f64("y").unwrap());
                        user_position.set_z(doc.get_f64("z").unwrap());
                        user_position
                    })
                    .collect();
                let mut reply = ConnectPositionResponse::new();
                reply.set_users(RepeatedField::from_vec(users));
                (reply, WriteFlags::default())
            });
            let f = resp
                .send_all(to_send)
                .map(|_| {})
                .map_err(|e| println!("failed : {:?}", e));
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
            "x": 0.0,
            "y": 0.0,
            "z": 0.0,
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
    let user_service = protos::multiplay_grpc::create_user(UserService { client: client.clone()});
    let multiplay_service = protos::multiplay_grpc::create_multiplay(MultiplayService { client: client.clone() });
    let host = env::var("RUST_GRPC_HOST").unwrap_or("127.0.0.1".to_string()); 
    let port = env::var("RUST_GRPC_PORT").unwrap_or("57601".to_string());
    let mut server = ServerBuilder::new(env)
        .register_service(user_service)
        .register_service(multiplay_service)
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
