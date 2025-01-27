Distributed challenges for maelstrom nodes


Big q was to how to gossip? 
When the node is initialised it needs to establish a neighbourhood using the initialised nodes sent via the network. 

Easy strat is to gossip to all nodes within the network. ``self.messages.clone()`` .
Better strat is to identify nodes in the netowrk. Then send to the nodes that the current node has in their neighbourhood 

Wouldn't waste too long on understanding this just good to know
 ``for n in &self.neighborhood {
let known_to_n = &self.known[n];
Message {
    src: self.node.clone(),
    dst: n.clone(),
    body: Body {
        id: None,
        in_reply_to: None,
        payload: Payload::Gossip {
            seen: self
                .messages
                .iter()
                .copied()
                .filter(|m| !known_to_n.contains(m))
                .collect(),
        },
    },
}
.send(&mut *output)
.with_context(|| format!("gossip to {}", n))?; ``
