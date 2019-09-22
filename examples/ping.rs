use std::net::Ipv4Addr;
use structopt::StructOpt;
use healthy::ping;

#[derive(StructOpt)]
struct Args {
    hosts: Vec<Ipv4Addr>,
}

fn main() {
    let args = Args::from_args();

    let status = ping::send(&args.hosts).unwrap();
    for host in status {
        println!("{:?}", host);
    }
}
