use clap::Parser;
use log::info;
pub mod server;
pub mod initiator;
pub mod queue;

#[derive(Parser, Debug)]
struct Args{
    #[arg(short, long)]
    address: String,
    #[arg(short, long)]
    server_port: u16,
    #[arg(short, long)]
    initiator_port: u16,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let address = args.address;
    let initiator_address = format!("{}:{}", address, args.initiator_port);
    let server = server::server::Server::new(address, args.server_port);
    info!("initiator address: {}", initiator_address);
    let mut initiator = initiator::initiator::Initiator::new(initiator_address);

    let res = tokio::join!(
        server.run(),
        initiator.run(),
    );
    match res {
        (Ok(_), Ok(_)) => (),
        (Err(e), _) => eprintln!("server error: {}", e),
        (_, Err(e)) => eprintln!("initiator error: {}", e),
    }


}
