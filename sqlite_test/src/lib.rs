use serde::{Deserialize, Serialize};

use uqbar_process_lib::{Address, ProcessId, Request, Response};
use uqbar_process_lib::kernel_types as kt;
use uqbar_process_lib::uqbar::process::standard as wit;

mod tester_types;
use tester_types as tt;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

// use bindings::component::uq_process::types::*;
// use bindings::{get_payload, Guest, print_to_terminal, receive, send_and_await_response, send_response};
use crate::sqlite_types::Deserializable;

mod sqlite_types;
use sqlite_types as sq;

const DB_NAME: &str = "foobar";

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
                tt::TesterRequest::KernelMessage(_) => {},
                tt::TesterRequest::GetFullMessage(_) => {},
                tt::TesterRequest::Run(_) => {
                    wit::print_to_terminal(0, "sqlite_test: a");
                    let sqlite_address = Address {
                        node: our.node.clone(),
                        process: ProcessId::new("sqlite", "sqlite", "uqbar"),
                    };

                    // New
                    wit::print_to_terminal(0, "sqlite_test: New 0");
                    let _ = Request::new()
                        .target(sqlite_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&sq::SqliteMessage::New {
                            db: DB_NAME.into()
                        })?)
                        .send_and_await_response(15)??;
                    wit::print_to_terminal(0, "sqlite_test: New done");

                    // Write
                    wit::print_to_terminal(0, "sqlite_test: Write 0");
                    let create_table = "CREATE TABLE person (id INTEGER PRIMARY KEY, name TEXT NOT NULL, data BLOB)".into();
                    let insert_data = "INSERT INTO person (name, data) VALUES (?1, ?2)".to_string();
                    let mut insert_data_vec = vec![
                        sq::SqlValue::Text("hello".into()),
                        sq::SqlValue::Blob(vec![1, 2, 3]),
                    ];
                    let insert_data_bytes = rmp_serde::to_vec(&insert_data_vec).unwrap();
                    let _ = Request::new()
                        .target(sqlite_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&sq::SqliteMessage::Write {
                            db: DB_NAME.into(),
                            statement: create_table,
                        })?)
                        .send_and_await_response(15)??;
                    wit::print_to_terminal(0, "sqlite_test: Write 1");
                    let _ = Request::new()
                        .target(sqlite_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&sq::SqliteMessage::Write {
                            db: DB_NAME.into(),
                            statement: insert_data,
                        })?)
                        .payload(wit::Payload {
                            mime: None,
                            bytes: insert_data_bytes,
                        })
                        .send_and_await_response(15)??;
                    wit::print_to_terminal(0, "sqlite_test: Write done");

                    // Read
                    wit::print_to_terminal(0, "sqlite_test: Read 0");
                    let select_data = "SELECT id, name, data FROM person".into();
                    let (_, response) = Request::new()
                        .target(sqlite_address.clone())?
                        .ipc_bytes(serde_json::to_vec(&sq::SqliteMessage::Read {
                            db: DB_NAME.into(),
                            query: select_data,
                        })?)
                        .send_and_await_response(15)??;
                    let payload = wit::get_payload().unwrap();
                    let payload = Vec::<Vec::<sq::SqlValue>>::from_serialized(&payload.bytes)?;

                    insert_data_vec.insert(0, sq::SqlValue::Integer(1));
                    if !payload.contains(&insert_data_vec)  {
                        return Err(anyhow::anyhow!(
                            "sqlite_test: Read gave unexpected value: {:?} not amongst {:?}",
                            insert_data_vec,
                            payload,
                        ));
                    }

                    wit::print_to_terminal(0, &format!("sqlite_test: Read done: {:?}\n{:?}", response, payload));

                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                },
                _ => {
                    wit::print_to_terminal(0, &format!("sqlite_test: b {:?}", ipc));
                },
            }

            Ok(())
        },
    }
}

struct Component;
impl Guest for Component {
    fn init(our: String) {
        wit::print_to_terminal(0, "sqlite_test: begin");

        let our = Address::from_str(&our).unwrap();

        loop {
            match handle_message(&our) {
                Ok(()) => {},
                Err(e) => {
                    wit::print_to_terminal(0, format!(
                        "sqlite_test: error: {:?}",
                        e,
                    ).as_str());

                    fail!("sqlite_test");
                },
            };
        }
    }
}
