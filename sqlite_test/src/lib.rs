use uqbar_process_lib::{Address, ProcessId, Request, Response};
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
                tt::TesterRequest::Run { .. } => {
                    wit::print_to_terminal(0, "sqlite_test: a");
                    let sqlite_address = Address {
                        node: our.node.clone(),
                        process: ProcessId::new(Some("sqlite"), "sqlite", "uqbar"),
                    };

                    // New
                    wit::print_to_terminal(0, "sqlite_test: New 0");
                    let _ = Request::new()
                        .target(sqlite_address.clone())
                        .ipc(serde_json::to_vec(&sq::SqliteMessage::New {
                            db: DB_NAME.into()
                        })?)
                        .send_and_await_response(5)?.unwrap();
                    wit::print_to_terminal(0, "sqlite_test: New done");

                    // Write
                    wit::print_to_terminal(0, "sqlite_test: Write 0");
                    let create_table = "CREATE TABLE person (id INTEGER PRIMARY KEY, name TEXT NOT NULL, data BLOB)".into();
                    let insert_data = "INSERT INTO person (name, data) VALUES (?1, ?2)".to_string();
                    let insert_data_json = serde_json::json!([
                        "hello",
                        format!("{}", base64::encode(vec![1, 2, 3])),
                    ]);
                    let insert_data_bytes = serde_json::to_vec(&insert_data_json)?;
                    let _ = Request::new()
                        .target(sqlite_address.clone())
                        .ipc(serde_json::to_vec(&sq::SqliteMessage::Write {
                            db: DB_NAME.into(),
                            statement: create_table,
                            tx_id: None,
                        })?)
                        .send_and_await_response(5)?.unwrap();
                    wit::print_to_terminal(0, "sqlite_test: Write 1");
                    let _ = Request::new()
                        .target(sqlite_address.clone())
                        .ipc(serde_json::to_vec(&sq::SqliteMessage::Write {
                            db: DB_NAME.into(),
                            statement: insert_data,
                            tx_id: None,
                        })?)
                        .payload(wit::Payload {
                            mime: None,
                            bytes: insert_data_bytes,
                        })
                        .send_and_await_response(5)?.unwrap();
                    wit::print_to_terminal(0, "sqlite_test: Write done");

                    // Read
                    wit::print_to_terminal(0, "sqlite_test: Read 0");
                    let select_data = "SELECT name, data FROM person".into();
                    let response = Request::new()
                        .target(sqlite_address.clone())
                        .ipc(serde_json::to_vec(&sq::SqliteMessage::Read {
                            db: DB_NAME.into(),
                            query: select_data,
                        })?)
                        .send_and_await_response(5)?.unwrap();
                    let payload = wit::get_payload().unwrap();
                    let serde_json::Value::Array(payload) = serde_json::from_slice(&payload.bytes)? else {
                        fail!("sqlite_test");
                    };
                    if payload.len() != 1 {
                        fail!("sqlite_test");
                    }
                    let Some(name) = payload[0].get("name") else {
                        fail!("sqlite_test");
                    };
                    let Some(data) = payload[0].get("data") else {
                        fail!("sqlite_test");
                    };
                    if name != &insert_data_json[0] {
                        fail!("sqlite_test");
                    }
                    if data != &insert_data_json[1] {
                        fail!("sqlite_test");
                    }

                    wit::print_to_terminal(0, &format!("sqlite_test: Read done: {:?}\n{:?}", response, payload));

                    Response::new()
                        .ipc(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
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

        wit::create_capability(
            &ProcessId::new(Some("sqlite"), "sqlite", "uqbar"),
            &"\"messaging\"".into(),
        );

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
