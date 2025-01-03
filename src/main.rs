use std::io::{self, stdout, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename = "type")]
    type_of_msg: String,
    msg_id: usize,
    echo: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Body")]
struct MessageResponse {
    src: String,
    dest: String,
    body: BodyResponse,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Body")]
struct BodyResponse {
    #[serde(rename = "type")]
    type_of_msg: String,
    // unique id identifier
    msg_id: Option<usize>,
    // msg id you are respending
    in_reply_to: Option<usize>,
    /*     echo: Option<String>, */
}

fn main() {
    let mut output = io::stdout().lock();
    let input = io::stdin().lock();

    let mut message = serde_json::Deserializer::from_reader(input).into_iter::<Message>();

    while let Some(message_serialised) = message.next() {
        tracing::debug!("hit i think");
        let next_msg = message_serialised;

        match next_msg {
            Ok(val) => {
                let msg_response = MessageResponse {
                    src: val.src,
                    dest: val.dest,
                    body: BodyResponse {
                        type_of_msg: val.body.type_of_msg,
                        msg_id: Some(val.body.msg_id),
                        in_reply_to: Some(val.body.msg_id),
                        /*                         echo: val.body.echo, */
                    },
                };
                serde_json::to_writer(&mut output, &msg_response);
                output.write_all(b"\n");
            }
            Err(err) => {
                output.write_all(format!("{}", err.to_string()).as_bytes());
            }
        }
    }
}
