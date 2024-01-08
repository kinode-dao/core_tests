use serde::{Deserialize, Serialize};

use uqbar_process_lib::{
    await_message,
    kv::{self, Kv},
    println,
    sqlite::{self, Sqlite},
    Address, Message, PackageId, ProcessId, Request, Response,
};

mod tester_types;
use tester_types as tt;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

mod key_value_types;
use key_value_types as kv;

const DB_NAME: &str = "foobar";

fn handle_message(our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response(_) => {
            unimplemented!()
        }
        Message::Request(Request { ipc, .. }) => {
            match serde_json::from_slice(&ipc)? {
                tt::TesterRequest::KernelMessage(_) => {}
                tt::TesterRequest::GetFullMessage(_) => {}
                tt::TesterRequest::Run { .. } => {
                    println!("key_value_test: a");

                    let kv = kv::open(our.package_id(), "test")?;
                    println!("key_value_test: open done");

                    let key = vec![1, 2, 3];
                    let value = vec![4, 5, 6];

                    // Write
                    println!("key_value_test: set key [1, 2, 3] value [4, 5, 6]");

                    kv.set(key.clone(), value, None)?;

                    println!"key_value_test: set done");

                    // Read
                    println!("key_value_test: get [1, 2, ]");
                    let return_value = kv.get(key, None)?;

                    if return_value != value {
                        fail!("key_value_test");
                    }

                    println!("key_value_test: get done");

                    Response::new()
                        .ipc(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                }
            }

            Ok(())
        }
    }
}

struct Component;
impl Guest for Component {
    fn init(our: String) {
        println!("key_value_test: begin");

        let our = Address::from_str(&our).unwrap();

        loop {
            match handle_message(&our) {
                Ok(()) => {}
                Err(e) => {
                    println!("key_value_test: error: {:?}", e);
                    fail!("key_value_test");
                }
            };
        }
    }
}
