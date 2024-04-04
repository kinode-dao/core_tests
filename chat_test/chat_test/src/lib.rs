use serde::{Deserialize, Serialize};

use kinode_process_lib::{await_message, call_init, print_to_terminal, Address, Message, ProcessId, Request, Response};

mod tester_types;
use tester_types as tt;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
});

#[derive(Debug, Serialize, Deserialize)]
enum ChatRequest {
    Send { target: String, message: String },
    History,
}

#[derive(Debug, Serialize, Deserialize)]
enum ChatResponse {
    History { messages: MessageArchive },
}

type MessageArchive = Vec<(String, String)>;

fn handle_message (our: &Address) -> anyhow::Result<()> {
    let message = await_message().unwrap();

    match message {
        Message::Response { .. } => { unimplemented!() },
        Message::Request { ref source, ref body, .. } => {
            if our.node != source.node {
                return Err(anyhow::anyhow!(
                    "rejecting foreign Message from {:?}",
                    source,
                ));
            }
            match serde_json::from_slice(body)? {
                tt::TesterRequest::KernelMessage(_) => {},
                tt::TesterRequest::GetFullMessage(_) => {},
                tt::TesterRequest::Run { input_node_names: node_names, .. } => {
                    print_to_terminal(0, "chat_test: a");
                    assert!(node_names.len() >= 2);
                    if our.node == node_names[0] {
                        // we are master node

                        let our_chat_address = Address {
                            node: our.node.clone(),
                            process: ProcessId::new(Some("chat"), "chat", "template.os"),
                        };
                        let their_chat_address = Address {
                            node: node_names[1].clone(),
                            process: ProcessId::new(Some("chat"), "chat", "template.os"),
                        };

                        // Send
                        print_to_terminal(0, "chat_test: b");
                        let message: String = "hello".into();
                        let _ = Request::new()
                            .target(our_chat_address.clone())
                            .body(serde_json::to_vec(&ChatRequest::Send {
                                target: node_names[1].clone(),
                                message: message.clone(),
                            })?)
                            .send_and_await_response(15)?.unwrap();

                        // Get history from receiver & test
                        print_to_terminal(0, "chat_test: c");
                        let response = Request::new()
                            .target(their_chat_address.clone())
                            .body(serde_json::to_vec(&ChatRequest::History)?)
                            .send_and_await_response(15)?.unwrap();
                        let Message::Response { ref body, .. } = response else { panic!("") };
                        let ChatResponse::History { messages } = serde_json::from_slice(body)? else {
                            fail!("chat_test");
                        };

                        let expected_messages = vec![(our.node.clone(), message)];

                        if messages != expected_messages {
                            fail!("chat_test");
                        }
                    }
                    Response::new()
                        .body(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                },
            }

            Ok(())
        },
    }
}

call_init!(init);
fn init(our: Address) {
    print_to_terminal(0, "begin");

    loop {
        match handle_message(&our) {
            Ok(()) => {},
            Err(e) => {
                print_to_terminal(0, format!(
                    "chat_test: error: {:?}",
                    e,
                ).as_str());

                fail!("chat_test");
            },
        };
    }
}
