use std::io::{self};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use state::State;

mod state;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    dest: String,
    body: Body,
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
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin().lock();
    let mut inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = io::stdout().lock();

    let mut state = State { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;

        state
            .step(input.clone(), &mut stdout)
            .context(format!("Step failed for input: {:?}", input))?;
    }

    Ok(())
}
