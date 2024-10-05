/// This is the main entry point for the blockchain application
use blockchain::models::client::Client;
use blockchain::start;
use clap::Parser;
use igd::search_gateway;
use local_ip_address::local_ip;
use std::net::{Ipv4Addr, SocketAddrV4};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port to listen on
    port: u16,

    /// The port of a peer node
    #[arg(short, long)]
    peer_port: Option<u16>,

    /// is client
    #[arg(short, long, action)]
    client: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let gateway = match search_gateway(Default::default()) {
        Ok(gateway) => gateway,
        Err(e) => {
            println!("[ERROR] Failed to find gateway: {}", e);
            return Ok(());
        }
    };

    if args.client {
        // start the client
        println!("[INFO] Starting client");
        let client = Client::new(args.port as u16);
        client.start().await;
        Ok(())
    } else {
        let external_port = 9487;
        let duration = 60;
        let local_ip = local_ip()
            .expect("Failed to get local IP address")
            .to_string();
        let local_ip = local_ip
            .parse::<Ipv4Addr>()
            .expect("failed to parse IP address");
        let local_ip = SocketAddrV4::new(local_ip, args.port);
        print!("local_ip: {:?}\n", local_ip);

        match gateway.add_port(
            igd::PortMappingProtocol::TCP,
            external_port,
            local_ip,
            duration,
            "YoutaCoin blockchain",
        ) {
            Ok(_) => {
                println!("[INFO] Port {} forwarded to {}", args.port, local_ip);
            }
            Err(e) => {
                println!("[ERROR] Failed to forward port: {}", e);
            }
        }
        start(args.port, args.peer_port).await;
        match gateway.remove_port(igd::PortMappingProtocol::TCP, external_port) {
            Ok(_) => {
                println!("[INFO] Port {} removed", args.port);
            }
            Err(e) => {
                println!("[ERROR] Failed to remove port: {}", e);
            }
        }
        Ok(())
    }
}
