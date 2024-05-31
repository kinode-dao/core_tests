use crate::kinode::process::tester::{Request as TesterRequest, Response as TesterResponse, FailResponse};
use kinode_process_lib::{await_message, call_init, sqlite::open, Address, Message, Response};

mod tester_lib;

wit_bindgen::generate!({
    path: "target/wit",
    world: "tester-sys-v0",
    generate_unused_types: true,
    additional_derives: [PartialEq, serde::Deserialize, serde::Serialize, process_macros::SerdeJsonInto],
});

fn handle_message(our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    if !message.is_request() {
        unimplemented!();
    }

    if our.node != message.source().node {
        return Err(anyhow::anyhow!(
            "rejecting foreign Message from {:?}",
            message.source(),
        ));
    }
    match message.body().try_into()? {
        TesterRequest::Run { .. } => {
            println!("sqlite_test: opening/creating db");
            let db = open(our.package_id(), "tester", None)?;

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
                .body(TesterResponse::Run(Ok(())))
                .send()
                .unwrap();
        }
    }

    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("begin");

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
