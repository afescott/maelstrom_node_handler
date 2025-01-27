use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::{self, BufRead, Write},
};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use state::Node;

mod state;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

impl<Payload> Message<Payload>
where
    Payload: Serialize + Debug,
{
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
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        }
    }

    pub fn send(&self, output: &mut impl Write) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *output, self).context("serialization response msg")?;
        output.write_all(b"\n").context("write newline")?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[derive(Serialize, Deserialize, Debug, Clone)]
enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
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
    Gossip {
        seen: HashSet<usize>,
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

//for multi node gossip, 2 types of message one for injected (network) and one that our nodes sent
enum Event<Payload> {
    // i.e. from network
    Message(Message<Payload>),
    //
    Injected(InjectedPayload),
}

enum InjectedPayload {
    Gossip,
}

fn main() -> anyhow::Result<()> {
    main_loop::<Message<Payload>>()?;

    Ok(())
}

fn main_loop<P: DeserializeOwned + Debug + Send + 'static>() -> anyhow::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock().lines();

    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("No init message received")
            .context("Failed to read init msg from reader")?,
    )
    .context("Init msg couldn't be deserialized")?;

    let InitPayload::Init { node_id, node_ids } = init_msg.body.payload else {
        panic!("first message should be init");
    };

    let mut init =
        Node::from_init(node_id, node_ids, tx.clone()).context("node initialisation failed")?;

    let init_resp = Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &init_resp).context("Echo serialisation")?;
    stdout
        .write_all(b"\n")
        .context("write newline else buffer doesn't work")?;

    //Dropping stdin means no buffered data is lost
    drop(stdin);

    let jh = std::thread::spawn(move || {
        let stdin = io::stdin().lock();

        for input in stdin.lines() {
            let line = input.context("Maelstrom input from STDIN could not be deserialized")?;

            let input: Message<Payload> =
                serde_json::from_str(&line).context("Maelstrom input could not be deserailized")?;
            if let Err(_) = tx.send(Event::Message(input)) {
                return Ok::<_, anyhow::Error>(());
            }
        }

        Ok(())
    });

    for input in rx {
        init.step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    //2 layers of joining a thread
    jh.join()
        .expect("stdin thread panicked")
        .context("stdin thread err'd")?;

    Ok(())
}
