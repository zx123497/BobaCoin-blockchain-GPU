/// This is the main entry point for the blockchain application
use blockchain::models::client::Client;
use blockchain::start;
use clap::Parser;
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
    if args.client {
        // start the client
        println!("[INFO] Starting client");
        let client = Client::new(args.port as u16);
        client.start().await;
        Ok(())
    } else {
        start(args.port, args.peer_port).await;
        Ok(())
    }
}
