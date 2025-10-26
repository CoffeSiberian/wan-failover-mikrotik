use std::io::Read;
use std::process::Command;

mod ssh_connect;

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

fn main() {
    let destination_ping = "one.one.one.one";
    let count = 2;
    let timeout_seg = 1;

    let ssh_connection = ssh_connect::connect_ssh();

    let ssh_session = match ssh_connection {
        Ok(_session) => _session,
        Err(e) => {
            eprintln!("Failed to establish SSH connection: {}", e);
            return;
        }
    };

    // Movistar
    let wan1 = ping_to_interface(destination_ping, "vlan50", count, timeout_seg);

    // Ingbell
    let wan2 = ping_to_interface(destination_ping, "vlan51", count, timeout_seg);

    // Mundo
    let wan3 = ping_to_interface(destination_ping, "vlan52", count, timeout_seg);

    if wan1 {
        let status = match get_status_route(&ssh_session, "LAN ROUTE 1") {
            Some(s) => s,
            None => {
                eprintln!("Fallo al obtener el estado de la ruta 'LAN ROUTE 1'");
                return;
            }
        };

        if !status {
            println!("WAN 1 Nuevamente activa");
            set_status_route(&ssh_session, "LAN ROUTE 1", true);
        }
    } else {
        eprintln!("WAN 1 Caída");
        set_status_route(&ssh_session, "LAN ROUTE 1", false);
    }

    if wan2 {
        let status = match get_status_route(&ssh_session, "LAN ROUTE 2") {
            Some(s) => s,
            None => {
                eprintln!("Fallo al obtener el estado de la ruta 'LAN ROUTE 2'");
                return;
            }
        };

        if !status {
            println!("WAN 2 Nuevamente activa");
            set_status_route(&ssh_session, "LAN ROUTE 2", true);
        }
    } else {
        eprintln!("WAN 2 Caída");
        set_status_route(&ssh_session, "LAN ROUTE 2", false);
    }

    if wan3 {
        let status_inv_1 = match get_status_route(&ssh_session, "Invitados 1") {
            Some(s) => s,
            None => {
                eprintln!("Fallo al obtener el estado de la ruta 'Invitados 1'");
                return;
            }
        };
        if !status_inv_1 {
            println!("WAN 3 Nuevamente activa");
            set_status_route(&ssh_session, "Invitados 1", true);
        }

        let status_inv_2 = match get_status_route(&ssh_session, "Invitados 2") {
            Some(s) => s,
            None => {
                eprintln!("Fallo al obtener el estado de la ruta 'Invitados 2'");
                return;
            }
        };
        if !status_inv_2 {
            println!("WAN 3 Nuevamente activa");
            set_status_route(&ssh_session, "Invitados 2", true);
        }
    } else {
        eprintln!("WAN 3 Caída");
        set_status_route(&ssh_session, "Invitados 1", false);
        set_status_route(&ssh_session, "Invitados 2", false);
    }
}
