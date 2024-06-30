use serde::{Deserialize, Serialize};
use std::fs::File;
#[derive(Serialize, Deserialize, Debug)]
struct Connect {
    len: u8,
    msg_type: u8,
    flags: u8,
    proto_id: u8,
    duration: u16,
    client_id: [u8; 32],
}
pub fn test_bincode() {
    let mut c = Connect {
        len: 8,
        msg_type: 0b0001,
        flags: 2,
        proto_id: 22,
        duration: 111,
        client_id: [0; 32],
    };
    c.client_id[0] = 2;
    c.client_id[1] = 3;
    c.client_id[2] = 4;
    let bytes = bincode::serialize(&c).unwrap();
    println!("{:?}", bytes);
    let decoded: Connect = bincode::deserialize(&bytes).unwrap();
    println!("{:#?}", decoded);
}
