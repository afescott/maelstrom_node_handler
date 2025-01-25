use std::{
    collections::{HashMap, HashSet},
    io::{StdoutLock, Write},
    sync::mpsc::Sender,
    thread::sleep,
    time::Duration,
};

use anyhow::{bail, Context};
use serde::Serialize;

use crate::{Body, Event, EventPayload, Message, Payload};

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
        tx: Sender<EventPayload>,
    ) -> anyhow::Result<Self> {
        //Required for multi node broadcasting
        std::thread::spawn(|| loop {
            sleep(Duration::from_millis(300));

            tx.send(Event::Injected(crate::InjectedPayload::Gossip));
        });

        Ok(Self {
            node_id,
            id: 1,
            known_nodes: nodes_in_cluster
                .into_iter()
                .map(|i| (i, HashSet::new()))
                .collect(),
            neighbourhood: Vec::new(),
            messages: HashSet::new(),
            tx,
        })
    }
    pub fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input {
            Event::Injected(payload) => match payload {
                crate::InjectedPayload::Gossip => todo!(),
            },
            Event::Message(input) => {
                let mut reply = input.clone().into_reply(Some(&mut self.id));
                match input.body.payload {
                    Payload::Echo { echo } => {
                        reply.body.payload = crate::Payload::EchoOk { echo };

                        reply.send(output);
                    }
                    //This should not happen
                    Payload::EchoOk { .. } => {}
                    Payload::Generate => {
                        reply.body.payload = Payload::GenerateOk {
                            guid: ulid::Ulid::new().to_string(),
                        };

                        reply.send(output);
                    }
                    Payload::GenerateOk { .. } => bail!("Should not generate ok?"),
                    Payload::Topology { mut topology } => {
                        self.neighbourhood = topology
                            .remove(&self.node_id)
                            .unwrap_or_else(|| panic!("Topology should contain {}", self.node_id));
                        reply.body.payload = Payload::TopologyOk;

                        reply.send(output);
                    }
                    Payload::TopologyOk => {}
                    Payload::Broadcast { message } => {
                        reply.body.payload = Payload::BroadcastOk;
                        self.messages.insert(message);

                        reply.send(output);
                    }
                    Payload::BroadcastOk => {}
                    Payload::Read => {
                        reply.body.payload = Payload::ReadOk {
                            messages: self.messages.clone(),
                        };

                        reply.send(output);
                    }
                    crate::Payload::ReadOk { messages } => {}
                }
            }
        }
        Ok(())
    }
}
