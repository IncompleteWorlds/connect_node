/**
 * Author:       Alberto Fernandez
 * Date:         19/06/2024
 * Description:  This clients connect to a Monero node and perform
 *               the handshake. After that, it dumps all received
 *               messages to a log file
 *
 *  NOTE: It does not work because the Monero P2P uses Boosts for
 * serializing the messages and it is mainly undocumented.
 * The crate I have used 'epee_encoding' does not work with long
 * messages.
 *
 */
use std::{
    fs::File,
    io::{ErrorKind, Write},
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use clap::{arg, value_parser, Command};

use chrono::prelude::*;
use epee_encoding::{from_bytes, to_bytes};
use protocol::{
    HandshakeRequest, HandshakeResponse, Header, P2PMessage, PayloadType, PeerListEntryBase,
    HANDSHAKE_REQUEST, HEADER_SIZE, NETWORK_STATE_REQUEST, PEER_ID_REQUEST, PING_REQUEST,
    STAT_INFO_REQUEST, SUPPORT_FLAGS_REQUEST, TIMED_SYNC_REQUEST,
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod network;
mod protocol;

fn write_log(in_file: &Option<File>, in_message: impl AsRef<str>) {
    let now = Utc::now();
    let buffer = format!(
        "{}: {}",
        now.to_rfc3339_opts(SecondsFormat::Millis, true),
        in_message.as_ref()
    );

    if let Some(mut f) = in_file.as_ref() {
        writeln!(f, "{}", buffer).unwrap();
    } else {
        println!("{}", buffer);
    }
}

async fn read_message(
    in_log_file: &Option<File>,
    in_connection: &mut TcpStream,
) -> Result<P2PMessage, io::Error> {
    let mut output_message: P2PMessage = P2PMessage::new();

    let mut response_msg_header_buffer: [u8; HEADER_SIZE as usize] = [0; HEADER_SIZE as usize];
    let mut bytes_read = match in_connection
        .read_exact(&mut response_msg_header_buffer)
        .await
    {
        Ok(b) => b,
        Err(e) => {
            write_log(in_log_file, format!("ERROR: Reading handshake header.{e}"));
            return Err(e);
        }
    };
    write_log(in_log_file, format!("Bytes read: {}", bytes_read));

    // Process the header
    let mut response_header: Header = Header::new();
    response_header.from_bytes(&response_msg_header_buffer);

    write_log(
        in_log_file,
        format!("Response header {:?}", response_header),
    );

    output_message.header = response_header;

    // Read the rest of the message
    write_log(
        in_log_file,
        format!(
            "response_header.msg_length: {}",
            output_message.header.msg_length
        ),
    );
    if output_message.header.msg_length <= 0 {
        write_log(
            in_log_file,
            format!(
                "ERROR: Reading handshake responser header.Incorrect length: {}",
                output_message.header.msg_length
            ),
        );
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Message length is zero",
        ));
    }
    let mut response_p2p_msg_buffer: Vec<u8> = Vec::new();

    response_p2p_msg_buffer.resize(output_message.header.msg_length as usize, 0);

    bytes_read = in_connection.read(&mut response_p2p_msg_buffer).await?;
    write_log(in_log_file, format!("{:x?}", response_p2p_msg_buffer));
    // println!("{}", response_p2p_msg_buffer.by_ref().escape_ascii());

    output_message.buffer = response_p2p_msg_buffer;

    Ok(output_message)
}

async fn do_handshake(
    in_config_file: Option<String>,
    in_log_file: &Option<File>,
    in_connection: &mut TcpStream,
) -> Result<(), io::Error> {
    // Create request
    let mut request: HandshakeRequest = HandshakeRequest::new();

    // Read from a config file, if so
    match in_config_file {
        Some(c) => {
            request.load_from_file(c);
        }
        None => {
            // Set Node data
            request.set_node_data();

            // TODO: Set payload data
            request.set_payload_data();
        }
    }

    // Serialize the request
    let request_msg_buffer = match to_bytes(&request) {
        Ok(m) => m,
        Err(e) => {
            write_log(
                in_log_file,
                format!("ERROR: Encoding Handshake request: {}", e),
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "ERROR: Serializing request",
            ));
        }
    };

    // Create Handshake request message
    let mut request_p2p_message = P2PMessage::new_command(HANDSHAKE_REQUEST);

    // Set the message length and encode as byte array
    request_p2p_message.header.msg_length = request_msg_buffer.len() as u64;
    request_p2p_message.buffer = request_msg_buffer;

    let p2p_message_buffer = request_p2p_message.to_bytes();

    write_log(
        in_log_file,
        format!(
            "Sending request: {} {:x?}",
            request_p2p_message.header.msg_length, request_p2p_message
        ),
    );
    write_log(in_log_file, format!("{:x?}", p2p_message_buffer));

    // Send Handshake request
    in_connection.write_all(&p2p_message_buffer).await?;

    // Read reply message
    let received_p2p_message: P2PMessage = read_message(in_log_file, in_connection).await?;

    write_log(
        in_log_file,
        format!("Buffer len: {}", received_p2p_message.buffer.len()),
    );
    let response: HandshakeResponse = match from_bytes(&received_p2p_message.buffer) {
        Ok(r) => r,
        Err(e) => {
            write_log(
                in_log_file,
                format!("ERROR: Decoding Handshake response: {}", e),
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "ERROR: Deserializing request",
            ));
        }
    };

    // Check parameters
    if response.node_data.network_id != request.node_data.network_id {
        write_log(
            in_log_file,
            format!("ERROR: Wrong network: {:x?}", response.node_data.network_id),
        );
        return Err(io::Error::new(io::ErrorKind::Other, "ERROR: Wrong network"));
    }

    // Read the list of peer entries
    let _list_peers = read_peer_list(in_log_file, &response);

    process_payload_data(in_log_file, &response.payload_data).unwrap();

    Ok(())
}

fn process_payload_data(
    in_log_file: &Option<File>,
    in_data: &PayloadType,
) -> Result<(), io::Error> {
    write_log(
        in_log_file,
        format!("Processing payload data: {:x?}", in_data),
    );
    Ok(())
}

fn read_peer_list(
    in_log_file: &Option<File>,
    _in_response: &HandshakeResponse,
) -> Vec<PeerListEntryBase> {
    let output_list: Vec<PeerListEntryBase> = Vec::new();

    write_log(in_log_file, format!("Reading peer list"));

    output_list
}

async fn process_message(
    in_log_file: Arc<Mutex<Option<File>>>,
    in_message: P2PMessage,
    out_end_flag: Arc<Mutex<bool>>,
) {
    match in_message.header.command {
        HANDSHAKE_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Handshake request"));
        }

        TIMED_SYNC_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Time sync request"));
        }

        PING_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Ping request"));
        }
        STAT_INFO_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Info request"));
        }

        NETWORK_STATE_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Network State request"));
        }

        PEER_ID_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Peer ID request"));
        }

        SUPPORT_FLAGS_REQUEST => {
            let f = in_log_file.lock().unwrap();
            write_log(&f, format!("Reply to Support Flags request"));
        }
        _ => {
            let f = in_log_file.lock().unwrap();
            write_log(
                &f,
                format!("ERROR: Unsupported command: {}", in_message.header.command),
            );
            *out_end_flag.lock().unwrap() = true;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), u32> {
    let matches = Command::new("Connect to Node")
        .version("1.0")
        .about("Connect to a Monero node in the specified network and perform the handshake")
        .arg(
            arg!(<ip_address> "Node IP Address")
                .required(true)
                .value_parser(value_parser!(Ipv4Addr)),
        )
        .arg(arg!(<port> "Node Port").required(true))
        .arg(
            arg!(
                -o --output <log_file> "Log file to record the debug messages"
            )
            .required(false),
        )
        .arg(
            arg!(
                -c --config <json_config_file> "Config file containing a Handshake Request parametes"
            )
            .required(false),
        )

        .get_matches();

    let node_ip_address = matches
        .get_one::<Ipv4Addr>("ip_address")
        .expect("Please, enter a Node IP");

    let tmp_node_port = matches
        .get_one::<String>("port")
        .expect("Please, enter a Node port");
    let node_port = tmp_node_port.parse::<u16>().unwrap_or(0);

    let node_ip_address = matches
        .get_one::<Ipv4Addr>("ip_address")
        .expect("Please, enter a Node IP");

    let log_file_name = matches.get_one::<String>("output");

    let config_file_name = matches.get_one::<String>("config");

    // println!("DEBUG: {} {}", node_ip_address, node_port);
    // if let Some(l) = log_file_name {
    //     println!("{}", l);
    // }

    // Open the log file
    let mut log_file: Option<File> = None;

    if let Some(l) = log_file_name {
        log_file = match File::create(l) {
            Ok(f) => Some(f),
            Err(e) => {
                println!("ERROR: Opening log file: {}", e);
                None
            }
        };
    }

    write_log(&log_file, "Connect to Node started");
    write_log(
        &log_file,
        format!("Connecting to: {}:{}", node_ip_address, node_port),
    );

    // Connect to the node
    let connection_string = format!("{}:{}", node_ip_address, node_port);

    let mut node_stream = match TcpStream::connect(connection_string).await {
        Ok(n) => n,
        Err(e) => {
            write_log(&log_file, format!("ERROR: Connecting to node: {}", e));
            return Err(1);
        }
    };

    write_log(&log_file, "Connected");

    let tmp_config: Option<String> = match config_file_name {
        Some(f) => Some(f.into()),
        None => None,
    };

    // Write data in the background
    // Do Handshake
    write_log(&log_file, "Performing handshake");
    do_handshake(tmp_config, &log_file, &mut node_stream)
        .await
        .unwrap();

    // Read message until Ctrl-C is pressed
    // let mut end_flag = false;
    let arc_end_flag: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let arc_log_file: Arc<Mutex<Option<File>>> = Arc::new(Mutex::new(log_file));

    while !*arc_end_flag.lock().unwrap() {
        let f = arc_log_file.lock().unwrap();
        let input_message: P2PMessage = match read_message(&f, &mut node_stream).await {
            Ok(m) => m,
            Err(e) => {
                write_log(&f, format!("ERROR: Reading message: {}", e));
                sleep(Duration::from_millis(500));
                continue;
            }
        };

        let tmp_log_file = arc_log_file.clone();
        let tmp_end_flag = arc_end_flag.clone();

        tokio::spawn(async {
            process_message(tmp_log_file, input_message, tmp_end_flag).await;
        })
        .await
        .unwrap();
    }

    // Close connection
    write_log(&arc_log_file.lock().unwrap(), "Closing connection");
    node_stream.shutdown().await.unwrap();

    Ok(())
}
