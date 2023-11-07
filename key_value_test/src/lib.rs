use serde::{Deserialize, Serialize};

use uqbar_process_lib::{Address, ProcessId, Request, Response};
use uqbar_process_lib::uqbar::process::standard as wit;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

mod key_value_types;
use key_value_types as kv;

struct Component;

const DB_NAME: &str = "foobar";

#[derive(Debug, Serialize, Deserialize)]
enum TesterRequest {
    Run,
}

#[derive(Debug, Serialize, Deserialize)]
enum TesterResponse {
    Pass,
    Fail,
}

fn handle_message (our: &Address) -> anyhow::Result<()> {
    let (source, message) = wit::receive().unwrap();

    if our.node != source.node {
        return Err(anyhow::anyhow!(
            "rejecting foreign Message from {:?}",
            source,
        ));
    }

    match message {
        wit::Message::Response(_) => { unimplemented!() },
        wit::Message::Request(wit::Request { ipc, .. }) => {
            match serde_json::from_slice(&ipc)? {
                TesterRequest::Run => {
                    wit::print_to_terminal(0, "key_value_test: a");
                    let key_value_address = Address {
                        node: our.node.clone(),
                        process: ProcessId::new("key_value", "key_value", "uqbar"),
                    };

                    let key = vec![1, 2, 3];
                    let value = vec![4, 5, 6];

                    // New
                    wit::print_to_terminal(0, "key_value_test: New 0");
                    let _ = Request::new()
                        .target(key_value_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&kv::KeyValueMessage::New {
                            db: DB_NAME.into()
                        })?)
                        .send_and_await_response(15)??;
                    wit::print_to_terminal(0, "key_value_test: New done");

                    // Write
                    wit::print_to_terminal(0, "key_value_test: Write 0");
                    let _ = Request::new()
                        .target(key_value_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&kv::KeyValueMessage::Write {
                            db: DB_NAME.into(),
                            key: key.clone(),
                        })?)
                        .payload_bytes(value.clone())
                        .send_and_await_response(15)??;
                    wit::print_to_terminal(0, "key_value_test: Write done");

                    // Read
                    wit::print_to_terminal(0, "key_value_test: Read 0");
                    let (_, response) = Request::new()
                        .target(key_value_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&kv::KeyValueMessage::Read {
                            db: DB_NAME.into(),
                            key: key.clone(),
                        })?)
                        .send_and_await_response(15)??;
                    let payload = wit::get_payload().unwrap();

                    if payload.bytes != value {
                        return Err(anyhow::anyhow!(
                            "key_value_test: Read gave unexpected value: {:?} not {:?}",
                            payload.bytes,
                            value,
                        ));
                    }

                    wit::print_to_terminal(0, &format!("key_value_test: Read done: {:?}\n{:?}", response, payload));

                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                }
            }

            Ok(())
        },
    }
}

impl Guest for Component {
    fn init(our: String) {
        wit::print_to_terminal(0, "key_value_test: begin");

        let our = Address::from_str(&our).unwrap();

        loop {
            match handle_message(&our) {
                Ok(()) => {},
                Err(e) => {
                    wit::print_to_terminal(0, format!(
                        "key_value_test: error: {:?}",
                        e,
                    ).as_str());

                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&TesterResponse::Fail).unwrap())
                        .send()
                        .unwrap();
                },
            };
        }
    }
}
