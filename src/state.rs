use std::io::{StdoutLock, Write};

use anyhow::{bail, Context};
use serde::Serialize;

use crate::{Body, Message};

pub struct Node {
    pub id: usize,
    pub node_id: String,
    pub nodes_in_cluster: Vec<String>,
    pub messages: Vec<String>,
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
            crate::Payload::Echo { echo } => {
                reply.body.payload = crate::Payload::EchoOk { echo };
                serde_json::to_writer(&mut *output, &reply).context("Echo serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            crate::Payload::Init { .. } => {
                /*                 input.body.payload = crate::Payload::InitOk; */

                serde_json::to_writer(&mut *output, &reply).context("Init serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            //This should not happen
            crate::Payload::InitOk { .. } => bail!("Error"),
            crate::Payload::EchoOk { .. } => {}
            crate::Payload::Generate => {
                /* input.body.payload = crate::Payload::GenerateOk {
                    guid: ulid::Ulid::new().to_string(),
                }; */

                serde_json::to_writer(&mut *output, &reply).context("Generate serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            crate::Payload::GenerateOk { .. } => bail!("Should not generate ok?"),
            crate::Payload::Topology { topology } => todo!(),
            crate::Payload::TopologyOk => todo!(),
            crate::Payload::Broadcast { message } => todo!(),
            crate::Payload::BroadcastOk => todo!(),
            crate::Payload::Read => todo!(),
            crate::Payload::ReadOk { messages } => todo!(),
        }
        Ok(())
    }
}
