# WAN Failover for MikroTik

A Rust-based monitoring tool that automatically manages WAN failover on MikroTik routers via SSH. The application monitors multiple WAN connections through ping tests and automatically enables/disables routing rules based on connection availability.

## Features

-   **Multi-WAN Monitoring**: Monitors up to 3 WAN connections simultaneously
-   **Automatic Failover**: Automatically enables/disables routes based on connectivity status
-   **SSH Integration**: Manages MikroTik router configuration via SSH
-   **ICMP Monitoring**: Uses ping tests to verify connectivity through specific network interfaces
-   **Compile-time Configuration**: Embeds SSH credentials securely at build time

## How It Works

The application performs the following operations:

1. Establishes an SSH connection to the MikroTik router
2. Pings a destination (default: one.one.one.one) through each WAN interface
3. Based on ping results, enables or disables corresponding routes in the MikroTik router
4. Manages routing rules for LAN and guest networks

## Prerequisites

-   **Rust**: Version 1.70 or later
-   **Linux System**: Required for network interface ping operations
-   **Network Access**: The host machine must have direct access to the WAN interfaces being monitored
-   **MikroTik Router**: With SSH access enabled and properly configured routes

## Environment Variables

Create a `.env` file in the project root with the following variables:

```bash
# MikroTik SSH Connection Settings
MT_SSH_HOST=192.168.88.1       # MikroTik router IP address
MT_SSH_PORT=22                 # SSH port (default: 22)
MT_SSH_USER=admin              # SSH username
MT_SSH_PASS=your_password      # SSH password
```

**Important**: The `.env` file is read at **compile time**, not runtime. After modifying the `.env` file, you must recompile the application.

### Security Note

-   Never commit the `.env` file to version control
-   Add `.env` to your `.gitignore` file
-   Consider using SSH key authentication in production environments

## Configuration

### Network Interfaces

By default, the application monitors these VLAN interfaces:

-   **WAN 1** (Movistar): `vlan50` → Controls route: `LAN ROUTE 1`
-   **WAN 2** (Ingbell): `vlan51` → Controls route: `LAN ROUTE 2`
-   **WAN 3** (Mundo): `vlan52` → Controls routes: `Invitados 1`, `Invitados 2`

To modify these settings, edit the `src/main.rs` file:

```rust
// Change interface names
let wan1 = ping_to_interface(destination_ping, "vlan50", count, timeout_seg);

// Change route comments
set_status_route(&ssh_session, "LAN ROUTE 1", true);
```

### Ping Settings

Current ping configuration:

-   **Destination**: `one.one.one.one` (Cloudflare DNS)
-   **Count**: 2 packets
-   **Timeout**: 1 second per packet

Modify these in the `main()` function if needed.

## MikroTik Router Configuration

### Required Routes

Your MikroTik router must have routes with specific comments that this tool will manage:

```routeros
# LAN Routes
/ip route add dst-address=0.0.0.0/0 gateway=<wan1-gateway> comment="LAN ROUTE 1"
/ip route add dst-address=0.0.0.0/0 gateway=<wan2-gateway> comment="LAN ROUTE 2"

# Guest Network Routes
/ip route add dst-address=0.0.0.0/0 gateway=<wan3-gateway> comment="Invitados 1"
/ip route add dst-address=0.0.0.0/0 gateway=<wan3-gateway> comment="Invitados 2"
```

### SSH Access

Enable SSH on your MikroTik router:

```routeros
/ip service enable ssh
/ip service set ssh port=22
```

Ensure the user account has sufficient permissions to modify routes.

## Building the Project

### Debug Build

```bash
cargo build
```

The executable will be located at: `target/debug/wan-failover-mikrotik`

### Release Build (Recommended)

```bash
cargo build --release
```

The optimized executable will be located at: `target/release/wan-failover-mikrotik`

### Build Notes

-   The project uses `vendored-openssl` to statically link OpenSSL, avoiding runtime dependencies
-   SSH credentials are embedded at compile time via the `build.rs` script
-   You must have a `.env` file present during compilation

## Running the Application

### Single Execution

```bash
# Debug build
./target/debug/wan-failover-mikrotik

# Release build
./target/release/wan-failover-mikrotik
```

### Running with Cron

For periodic execution, add to crontab:

```bash
# Run every minute
* * * * * /path/to/wan-failover-mikrotik >> /var/log/wan-failover.log 2>&1
```

## Dependencies

### Runtime Dependencies

-   OpenSSL libraries (statically linked via vendored feature)

### Rust Dependencies

-   `ssh2`: SSH client implementation with vendored OpenSSL
-   `openssl`: Vendored OpenSSL bindings

### Build Dependencies

-   `dotenv-build`: Loads environment variables at compile time

## Project Structure

```
.
├── build.rs              # Compile-time .env loader
├── Cargo.toml            # Project dependencies and metadata
├── README.md             # This file
├── version               # Version file
├── src/
│   ├── main.rs          # Main application logic and WAN monitoring
│   └── ssh_connect.rs   # SSH connection handling
└── target/              # Build artifacts (generated)
```
