use kinode_process_lib::{await_message, call_init, sqlite::open, Address, Message, Response};

mod tester_types;
use tester_types as tt;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

fn handle_message(our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { .. } => {
            unimplemented!()
        }
        Message::Request {
            ref source,
            ref body,
            ..
        } => {
            if our.node != source.node {
                return Err(anyhow::anyhow!(
                    "rejecting foreign Message from {:?}",
                    source,
                ));
            }
            match serde_json::from_slice(body)? {
                tt::TesterRequest::KernelMessage(_) => {}
                tt::TesterRequest::GetFullMessage(_) => {}
                tt::TesterRequest::Run { .. } => {
                    println!("sqlite_test: opening/creating db");
                    let db = open(our.package_id(), "tester")?;

                    println!("sqlite_test: create table");
                    let create_table_statement =
                        "CREATE TABLE kinos (id INTEGER PRIMARY KEY, name TEXT NOT NULL);"
                            .to_string();

                    db.write(create_table_statement, vec![], None)?;

                    println!("sqlite_test: insert rows");

                    let insert_statement =
                        "INSERT INTO kinos (name) VALUES (?), (?), (?);".to_string();
                    let params = vec![
                        serde_json::Value::String("cinecafe".to_string()),
                        serde_json::Value::String("lumiere".to_string()),
                        serde_json::Value::String("dertoten".to_string()),
                    ];

                    db.write(insert_statement, params, None)?;

                    println!("sqlite_test: select rows");
                    let select_statement = "SELECT * FROM kinos;".to_string();

                    let rows = db.read(select_statement, vec![]).unwrap();

                    assert_eq!(rows.len(), 3);

                    Response::new()
                        .body(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                }
            }

            Ok(())
        }
    }
}

call_init!(init);

fn init(our: Address) {
    println!("sqlite_test: begin");

    loop {
        match handle_message(&our) {
            Ok(()) => {}
            Err(e) => {
                println!("sqlite_test: error: {:?}", e);

                fail!("sqlite_test");
            }
        };
    }
}
