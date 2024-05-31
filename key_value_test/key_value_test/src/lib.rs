use crate::kinode::process::tester::{Request as TesterRequest, Response as TesterResponse, FailResponse};
use kinode_process_lib::{await_message, call_init, kv::open, Address, Response};

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
            println!("kv_test: opening/creating db");
            let db = open(our.package_id(), "tester", None)?;

            println!("kv_test: set & get & delete");
            db.set(b"foo", b"bar", None)?;

            let value = db.get(b"foo")?;
            assert_eq!(&value, b"bar");

            db.delete(b"foo", None)?;

            let value = db.get(b"foo");

            assert!(value.is_err());

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
                println!("kv_test: error: {:?}", e);

                fail!("kv_test");
            }
        };
    }
}
