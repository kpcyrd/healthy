use crate::errors::*;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::Command;


pub fn send(hosts: &[Ipv4Addr]) -> Result<HashMap<Ipv4Addr, bool>> {
    let mut args = vec![
        String::from("-c2"), // pings to send
        String::from("-t500"), // wait for response per ping
        String::from("-p10"), // delay between pings
    ];

    for host in hosts {
        args.push(host.to_string());
    }

    let output = Command::new("fping")
        .args(&args)
        .output()
        .context("failed to execute fping")?;
    let output = String::from_utf8(output.stdout)?;

    let mut status = HashMap::new();
    for line in output.split('\n') {
        // mark hosts with pongs as online
        if let Some(host) = line.split(' ').next() {
            if let Ok(host) = host.parse() {
                status.insert(host, true);
            }
        }
    }

    // mark remaining hosts as offline
    for host in hosts {
        if status.get(host).is_none() {
            info!("{:?} is offline", host);
            status.insert(*host, false);
        }
    }

    Ok(status)
}
