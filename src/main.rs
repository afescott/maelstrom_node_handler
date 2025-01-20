use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead, Write},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use state::Node;

mod state;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

impl Message {
    fn into_reply(self, id: Option<&mut usize>) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                id: id.map(|id| {
                    let mid = *id;
                    *id += 1;
                    mid
                }),
                /*                 Some(self.id), */
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        //other id names
        #[serde(rename = "id")]
        guid: String,
    },
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

pub enum EventPayload {
    Broadcast(Payload),
    //I.e. from network
    Injected(Payload),
}

fn main() -> anyhow::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let stdin = io::stdin().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = io::stdout().lock();

    let init_msg = inputs
        .next()
        .expect("Value")
        .context("No init received")
        .context("Init msg couldn't be deserialized")?;

    let Payload::Init { node_id, node_ids } = init_msg.body.payload else {
        panic!("first message should be init");
    };

    let init_resp = Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Body {
            id: init_msg.body.id,
            in_reply_to: init_msg.body.id,
            payload: crate::Payload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &init_resp).context("Echo serialisation")?;
    stdout
        .write_all(b"\n")
        .context("write newline else buffer doesn't work")?;

    let mut init = Node::from_init(node_id, node_ids, 0, tx);

    //Have to do this as it's not send
    drop(stdin);

    let jh = std::thread::spawn(move || {
        let stdin = io::stdin().lock();

        for input in stdin.lines() {
            let input = input.context("Maelstrom input from STDIN could not be deserialized")?;

            if let Payload::Init {
                ref node_id,
                ref node_ids,
            } = input.body.payload
            {
                let mut init = Node::from_init(
                    node_id.to_string(),
                    node_ids.to_vec(),
                    input.body.id.unwrap(),
                    tx,
                );
            }

            init.step(input.clone(), &mut stdout)
                .context(format!("Step failed for input: {:?}", input))?;
        }

        Ok(())
    });

    for input in rx {}

    Ok(())
}
