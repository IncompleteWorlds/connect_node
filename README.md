## P2P Node Handshaking

Alberto Fernandez

# Tasks
- Pick a publicly available P2P node (e.g. a blockchain one) implementation -
which itself doesn't need to be written in Rust
- and write a network handshake for it in Rust,
- and instructions on how to test it.

# Requirements

- Both the target node and the handshake code should compile at least on Linux.
- The solution has to perform a full protocol-level (post-TCP/etc.) handshake with the target node.
- The provided instructions should include information on how to verify that the handshake has concluded.
- The solution can not depend on the code of the target node (but it can share some of its dependencies).
- The submitted code can not reuse entire preexisting handshake implementations like libp2p_noise/XX.

# Non-requirements

- A lot of parameters can potentially be exchanged during the handshake, but only the mandatory ones need to be included.
- The solution can ignore any post-handshake traffic from the target node, and it doesn't have to keep the connection alive.

# Implement Handshake

The selected P2P is Monero. It uses the LEVIN protocol for its communications.
It a nutshell, messages are composed of a header and payload data. Where, the 
payload data depends on the type of message.


## Execution
The SW can be run with:
```sh
$ cargo run -- --help

$ cargo run  -- 18.132.93.91 28080   -o node_log.txt
```
It will start the handshake process with the selected node (IP). 
It is recommended to use the TestNet for testing purposes:

  176.9.0.187:28080
  51.79.173.165:28080
  192.99.8.110:28080
  37.187.74.171:28080
  77.172.183.193:28080

  Rhino test node: 18.132.93.91
    
However, it does not finish due to the undocumented 
serialization protocol.
The output of the process will be stored in the `node_log.txt` file.

# Testing

The SW can / could be tested in two ways:

- *Cargo test*. A set of test cases can be defined alongside of
the code and then executed with:
```sh
$ cargo test
```
For example, there are a couple of tests in `protocol.rs`

- *Fake server*. Developing a small python application (or using any other
script language) that listen for commands and send predefined answer. 
They could capture using a modified version of the Monerod server application.
In this way, for each message a predefined answer could be sent; either hard-coded
or read from a file



  

