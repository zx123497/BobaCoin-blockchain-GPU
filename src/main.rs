use blockchain::start;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port to listen on
    port: u32,

    /// The port of a peer node
    #[arg(short, long, default_value_t = 0)]
    peer_port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.peer_port == 0 {
        start(args.port, None).await;
    } else {
        start(args.port, Some(args.peer_port)).await;
    }

    Ok(())
}
