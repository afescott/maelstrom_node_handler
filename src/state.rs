use std::io::{StdoutLock, Write};

use anyhow::{bail, Context};
use serde::Serialize;

use crate::{Body, Message, Payload};

pub struct Node {
    pub id: usize,
    pub node_id: String,
    pub nodes_in_cluster: Vec<String>,
    pub messages: Vec<usize>,
}

impl Node {
    pub fn from_init(node_id: String, nodes_in_cluster: Vec<String>) -> Self {
        Self {
            node_id,
            id: 1,
            nodes_in_cluster,
            messages: Vec::new(),
        }
    }
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.clone().into_reply(Some(&mut self.id));
        match input.body.payload {
            Payload::Echo { echo } => {
                reply.body.payload = crate::Payload::EchoOk { echo };
                serde_json::to_writer(&mut *output, &reply).context("Echo serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            Payload::Init { .. } => {
                /*                 input.body.payload = crate::Payload::InitOk; */

                serde_json::to_writer(&mut *output, &reply).context("Init serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            //This should not happen
            Payload::InitOk { .. } => bail!("Error"),
            Payload::EchoOk { .. } => {}
            Payload::Generate => {
                reply.body.payload = Payload::GenerateOk {
                    guid: ulid::Ulid::new().to_string(),
                };

                serde_json::to_writer(&mut *output, &reply).context("Generate serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            Payload::GenerateOk { .. } => bail!("Should not generate ok?"),
            Payload::Topology { topology } => {
                reply.body.payload = Payload::TopologyOk;

                serde_json::to_writer(&mut *output, &reply).context("Topology serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            Payload::TopologyOk => {}
            Payload::Broadcast { message } => {
                reply.body.payload = Payload::BroadcastOk;
                self.messages.push(message);

                serde_json::to_writer(&mut *output, &reply).context("Broadcast serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            Payload::BroadcastOk => {}
            Payload::Read => {
                reply.body.payload = Payload::ReadOk {
                    messages: self.messages.clone(),
                };

                serde_json::to_writer(&mut *output, &reply).context("Read serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            crate::Payload::ReadOk { messages } => {}
        }
        Ok(())
    }
}
