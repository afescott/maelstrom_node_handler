use std::io::{StdoutLock, Write};

use anyhow::{bail, Context};
use serde::Serialize;

use crate::{Body, Message};

pub struct State {
    pub id: usize,
}

impl State {
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            crate::Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: crate::Payload::EchoOk { echo },
                    },
                };

                serde_json::to_writer(&mut *output, &reply).context("Echo serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;

                self.id += 1;
            }
            crate::Payload::Init { .. } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: crate::Payload::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply).context("Init serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;

                self.id += 1;
            }
            //This should not happen
            crate::Payload::InitOk { .. } => bail!("Error"),
            crate::Payload::EchoOk { .. } => {}
            crate::Payload::Generate => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: crate::Payload::GenerateOk {
                            guid: ulid::Ulid::new().to_string(),
                        },
                    },
                };

                serde_json::to_writer(&mut *output, &reply).context("Generate serialisation")?;
                output
                    .write_all(b"\n")
                    .context("write newline else buffer doesn't work")?;

                self.id += 1;
            }
            crate::Payload::GenerateOk { .. } => bail!("Should not generate ok?"),
        }
        Ok(())
    }
}
