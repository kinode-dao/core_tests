use serde::{Deserialize, Serialize};

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
                tt::TesterRequest::Run(node_names) => {
                    wit::print_to_terminal(0, "chat_test: a");
                    assert!(node_names.len() >= 2);
                    if our.node == node_names[0] {
                        // we are master node

                        let our_chat_address = Address {
                            node: our.node.clone(),
                            process: ProcessId::new("chat", "chat", "uqbar"),
                        };
                        let their_chat_address = Address {
                            node: node_names[1].clone(),
                            process: ProcessId::new("chat", "chat", "uqbar"),
                        };

                        // Send
                        wit::print_to_terminal(0, "chat_test: b");
                        let message: String = "hello".into();
                        let _ = Request::new()
                            .target(our_chat_address.clone())?
                            .ipc_bytes(serde_json::to_vec(&ChatRequest::Send {
                                target: node_names[1].clone(),
                                message: message.clone(),
                            })?)
                            .send_and_await_response(15)??;

                        // Get history from receiver & test
                        wit::print_to_terminal(0, "chat_test: c");
                        let (_, response) = Request::new()
                            .target(their_chat_address.clone())?
                            .ipc_bytes(serde_json::to_vec(&ChatRequest::History)?)
                            .send_and_await_response(15)??;
                        let wit::Message::Response((response, _)) = response else { panic!("") };
                        let ChatResponse::History { messages } = serde_json::from_slice(&response.ipc)? else { panic!("") };

                        let expected_messages = vec![(our.node.clone(), message)];

                        if messages != expected_messages {
                            fail!("chat_test");
                        }
                    }
                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&tt::TesterResponse::Pass).unwrap())
                        .send()
                        .unwrap();
                },
            }

            Ok(())
        },
    }
}


struct Component;
impl Guest for Component {
    fn init(our: String) {
        wit::print_to_terminal(0, "chat_test: begin");

        let our = Address::from_str(&our).unwrap();

        loop {
            match handle_message(&our) {
                Ok(()) => {},
                Err(e) => {
                    wit::print_to_terminal(0, format!(
                        "chat_test: error: {:?}",
                        e,
                    ).as_str());

                    fail!("chat_test");
                },
            };
        }
    }
}
