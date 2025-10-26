use std::io::Read;
use std::process::Command;

mod ssh_connect;

const DEFAULT_PING_DESTINATION: &str = "one.one.one.one";
const DEFAULT_PING_COUNT: u32 = 2;
const DEFAULT_PING_TIMEOUT: u64 = 1;

struct WanInterface<'a> {
    name: &'a str,
    vlan: &'a str,
    routes: Vec<&'a str>,
}

fn set_status_route(ssh_session: &ssh2::Session, comment: &str, enable: bool) {
    let command = if enable {
        format!("/ip route enable [find where comment=\"{}\"]", comment)
    } else {
        format!("/ip route disable [find where comment=\"{}\"]", comment)
    };

    let mut channel = ssh_session.channel_session().unwrap();
    match channel.exec(&command) {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn get_status_route(ssh_session: &ssh2::Session, comment: &str) -> Option<bool> {
    let command = format!(
        ":put [/ip route get [find where comment=\"{}\"] disabled]",
        comment
    );

    let mut channel = ssh_session.channel_session().ok()?;
    channel.exec(&command).ok()?;

    let mut s = String::new();
    channel.read_to_string(&mut s).ok()?;

    let trimmed = s.trim();
    if trimmed == "true" {
        Some(false)
    } else if trimmed == "false" {
        Some(true)
    } else {
        None
    }
}

fn ping_to_interface(destination: &str, interface: &str, count: u32, timeout_seg: u64) -> bool {
    match Command::new("ping")
        .arg("-I")
        .arg(interface)
        .arg("-c")
        .arg(count.to_string())
        .arg("-W")
        .arg(timeout_seg.to_string())
        .arg(destination)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn check_and_update_wan(
    ssh_session: &ssh2::Session,
    wan: &WanInterface,
    destination: &str,
    count: u32,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let is_up = ping_to_interface(destination, wan.vlan, count, timeout);

    for route in &wan.routes {
        let status = get_status_route(ssh_session, route)
            .ok_or_else(|| format!("Failed to get route status: {}", route))?;

        if is_up && !status {
            println!("{} Nuevamente activa", wan.name);
            set_status_route(ssh_session, route, true);
        } else if !is_up {
            eprintln!("{} Caída", wan.name);
            set_status_route(ssh_session, route, false);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ssh_session = ssh_connect::connect_ssh()?;

    let wans = vec![
        WanInterface {
            name: "WAN 1",
            vlan: "vlan50",
            routes: vec!["LAN ROUTE 1"],
        },
        WanInterface {
            name: "WAN 2",
            vlan: "vlan51",
            routes: vec!["LAN ROUTE 2"],
        },
        WanInterface {
            name: "WAN 3",
            vlan: "vlan52",
            routes: vec!["Invitados 1", "Invitados 2"],
        },
    ];

    for wan in wans {
        match check_and_update_wan(
            &ssh_session,
            &wan,
            DEFAULT_PING_DESTINATION,
            DEFAULT_PING_COUNT,
            DEFAULT_PING_TIMEOUT,
        ) {
            Ok(_) => {}
            Err(e) => eprintln!("Error processing {}: {}", wan.name, e),
        }
    }

    Ok(())
}
