use std::io::StdoutLock;

use serde::Serialize;

use crate::{Body, Message};

pub struct State {
    pub id: usize,
}

impl State {
    pub fn step(&mut self, input: Message, output: &mut serde_json::Serializer<StdoutLock>) {
        /* match input.body.payload {
            crate::Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: crate::Payload::EchOk { echo },
                    },
                };

                reply.serialize(output);

                self.id += 1;
            }
            _ => todo!(),
        } */
    }
}
