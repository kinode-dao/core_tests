use kinode_process_lib::{await_message, call_init, kv::open, Address, Message, Response};

mod tester_types;
use tester_types as tt;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
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
