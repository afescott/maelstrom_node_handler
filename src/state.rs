use std::{
    collections::{HashMap, HashSet},
    io::{StdoutLock, Write},
    sync::mpsc::Sender,
};

use anyhow::{bail, Context};
use serde::Serialize;

use crate::{Body, EventPayload, Message, Payload};

pub struct Node {
    //current id of the node
    pub id: usize,
    //node unique identifier
    pub node_id: String,
    // nodes in the topology (neighbourhood)? and the messages this node is aware of
    pub known_nodes: HashMap<String, HashSet<usize>>,
    // Each of the messages received by the node
    pub messages: HashSet<usize>,
    //nodes we gossip with
    pub neighbourhood: Vec<String>,
    //way to broadcast to other nodes
    tx: Sender<EventPayload>,
}

impl Node {
    pub fn from_init(
        node_id: String,
        nodes_in_cluster: Vec<String>,
        id: usize,
        tx: Sender<EventPayload>,
    ) -> Self {
        Self {
            node_id,
            id: id + 1,
            known_nodes: nodes_in_cluster
                .into_iter()
                .map(|i| (i, HashSet::new()))
                .collect(),
            neighbourhood: Vec::new(),
            messages: HashSet::new(),
            tx,
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
            Payload::Topology { mut topology } => {
                self.neighbourhood = topology
                    .remove(&self.node_id)
                    .unwrap_or_else(|| panic!("Topology should contain {}", self.node_id));
                reply.body.payload = Payload::TopologyOk;

                serde_json::to_writer(&mut *output, &reply).context("Topology serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;
            }
            Payload::TopologyOk => {}
            Payload::Broadcast { message } => {
                reply.body.payload = Payload::BroadcastOk;
                self.messages.insert(message);

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
