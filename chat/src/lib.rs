use serde::{Serialize, Deserialize};

use uqbar_process_lib::{Address, ProcessId, Request, Response};
use uqbar_process_lib::uqbar::process::standard as wit;

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
    Ack,
    History { messages: MessageArchive },
}

type MessageArchive = Vec<(String, String)>;

fn handle_message (
    our: &Address,
    message_archive: &mut MessageArchive,
) -> anyhow::Result<()> {
    let (source, message) = wit::receive().unwrap();

    match message {
        wit::Message::Response(_) => {
            wit::print_to_terminal(0, &format!("chat: unexpected Response: {:?}", message));
            panic!("");
        },
        wit::Message::Request(wit::Request { ref ipc, .. }) => {
            match serde_json::from_slice(ipc)? {
                ChatRequest::Send { ref target, ref message } => {
                    wit::print_to_terminal(0, "chat: a");
                    if target == &our.node {
                        wit::print_to_terminal(0, "chat: b");
                        message_archive.push((source.node.clone(), message.clone()));
                    } else {
                        wit::print_to_terminal(0, "chat: c");
                        let _ = Request::new()
                            .target(wit::Address {
                                node: target.clone(),
                                process: ProcessId::from_str("chat:chat:uqbar")?,
                            })?
                            .ipc_bytes(ipc.clone())
                            .send_and_await_response(5)
                            .unwrap();
                    }
                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&ChatResponse::Ack).unwrap())
                        .send()
                        .unwrap();
                },
                ChatRequest::History => {
                    wit::print_to_terminal(0, "chat: d");
                    Response::new()
                        .ipc_bytes(serde_json::to_vec(&ChatResponse::History {
                            messages: message_archive.clone(),
                        }).unwrap())
                        .send()
                        .unwrap();
                },
            }
        },
    }
    wit::print_to_terminal(0, "chat: e");
    Ok(())
}

struct Component;
impl Guest for Component {
    fn init(our: String) {
        wit::print_to_terminal(0, "chat: begin");

        let our = Address::from_str(&our).unwrap();
        let mut message_archive: MessageArchive = Vec::new();

        loop {
            match handle_message(&our, &mut message_archive) {
                Ok(()) => {},
                Err(e) => {
                    wit::print_to_terminal(0, format!(
                        "chat: error: {:?}",
                        e,
                    ).as_str());
                },
            };
        }
    }
}