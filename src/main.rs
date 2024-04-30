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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    start(args.port, args.peer_port).await;
    Ok(())
}
