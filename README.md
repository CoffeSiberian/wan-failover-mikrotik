# WAN Failover for MikroTik

A Rust-based monitoring tool that automatically manages WAN failover on MikroTik routers via SSH. The application monitors multiple WAN connections through ping tests and automatically enables/disables routing rules based on connection availability.

## Features

-   **Multi-WAN Monitoring**: Monitors up to 3 WAN connections simultaneously
-   **Automatic Failover**: Automatically enables/disables routes based on connectivity status
-   **Continuous Monitoring**: Runs as a persistent service with automatic reconnection
-   **SSH Integration**: Manages MikroTik router configuration via SSH
-   **ICMP Monitoring**: Uses ping tests to verify connectivity through specific network interfaces
-   **Docker Integration**: Automatically restarts Docker containers when specific WANs change status
-   **Compile-time Configuration**: Embeds SSH credentials securely at build time

## How It Works

The application runs as a continuous monitoring service that performs the following operations:

1. Establishes an SSH connection to the MikroTik router
2. Iterates through configured WAN interfaces with their associated routes
3. Pings a destination (default: one.one.one.one) through each WAN interface's VLAN
4. Checks the current status of each route in the MikroTik router
5. Based on ping results, automatically enables or disables corresponding routes
6. When WAN 2 status changes, automatically restarts the Docker container (if configured)
7. Logs WAN status changes (activation/deactivation) to the console
8. Waits 5 seconds and repeats the monitoring cycle
9. Automatically reconnects in case of SSH connection errors

## Prerequisites

-   **Rust**: Version 1.70 or later
-   **Linux System**: Required for network interface ping operations
-   **Docker** (Optional): Required only if Docker container restart integration is enabled
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

-   **WAN 1**: `vlan50` → Controls route: `LAN ROUTE 1`
-   **WAN 2**: `vlan51` → Controls route: `LAN ROUTE 2`
-   **WAN 3**: `vlan52` → Controls routes: `Invitados 1`, `Invitados 2`

To modify these settings, edit the `wans` vector in `src/main.rs`:

```rust
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
```

Each `WanInterface` struct contains:

-   `name`: Descriptive name for logging purposes
-   `vlan`: VLAN interface to monitor via ping
-   `routes`: List of MikroTik route comments to enable/disable

### Ping Settings

Current ping configuration (defined as constants in `src/main.rs`):

-   **Destination**: `one.one.one.one` (Cloudflare DNS) - `DEFAULT_PING_DESTINATION`
-   **Count**: 2 packets - `DEFAULT_PING_COUNT`
-   **Timeout**: 1 second per packet - `DEFAULT_PING_TIMEOUT`

Modify these constants at the top of the file if needed:

```rust
const DEFAULT_PING_DESTINATION: &str = "one.one.one.one";
const DEFAULT_PING_COUNT: u32 = 2;
const DEFAULT_PING_TIMEOUT: u64 = 1;
```

### Monitoring Cycle

The application performs monitoring checks every **5 seconds**. This interval is defined in the `main()` function:

```rust
fn main() {
    loop {
        match main_res() {
            Ok(_) => {}
            Err(e) => {
                println!("{}", log_time(&format!("Error in main loop: {}", e)));
            }
        }

        thread::sleep(Duration::from_secs(5));  // Adjust this value to change monitoring frequency
    }
}
```

To change the monitoring frequency, modify the `Duration::from_secs(5)` value in the main loop.

### Docker Integration

The application can automatically restart Docker containers when specific WAN connections change status. This is useful for services that need to reconnect or rebind after a WAN failover event.

**Docker Container Name** (defined as constant in `src/main.rs`):

-   **Container**: `playit-playit-1` - `DEFAULT_CONTAINER_NAME`

By default, the application restarts the Docker container when **WAN 2** status changes (either goes up or down). To modify this behavior, edit the `check_and_update_wan()` function in `src/main.rs`:

```rust
// In check_and_update_wan function
if is_up && !status {
    println!("{}", log_time(&format!("{} Nuevamente activa", wan.name)));
    set_status_route(ssh_session, route, true);
    if wan.name == "WAN 2" {  // Change this condition as needed
        docker_playit_restart();
    }
} else if !is_up && status {
    println!("{}", log_time(&format!("{} Caída", wan.name)));
    set_status_route(ssh_session, route, false);
    if wan.name == "WAN 2" {  // Change this condition as needed
        docker_playit_restart();
    }
}
```

**Note**: The host system must have Docker installed and the application user must have permission to execute `docker restart` commands.

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

### Manual Execution

The application runs as a continuous monitoring service with a 5-second cycle:

```bash
# Debug build
./target/debug/wan-failover-mikrotik

# Release build
./target/release/wan-failover-mikrotik
```

The application will:

-   Monitor all configured WANs every 5 seconds
-   Automatically reconnect if the SSH connection is lost
-   Log status changes to stdout
-   Run indefinitely until manually stopped (Ctrl+C)

### Running as a System Service

For production use, it's recommended to run the application as a systemd service. A sample service file (`wan-failover.service`) is included in the repository:

```bash
# Copy the service file to systemd directory
sudo cp wan-failover.service /etc/systemd/system/

# Reload systemd daemon
sudo systemctl daemon-reload

# Enable the service to start on boot
sudo systemctl enable wan-failover

# Start the service
sudo systemctl start wan-failover

# Check service status
sudo systemctl status wan-failover

# View logs
sudo journalctl -u wan-failover -f
```

**Note**: Make sure to update the paths in the service file to match your installation directory.

## Error Handling and Recovery

The application is designed for continuous operation with automatic recovery:

-   **SSH Connection Errors**: If the SSH connection fails or is lost, the error is logged and the application automatically attempts to reconnect in the next monitoring cycle (5 seconds)
-   **Route Processing Errors**: If an individual WAN check fails, it's logged but other WANs continue to be monitored
-   **Docker Command Errors**: Failed Docker restart attempts are logged but don't interrupt the monitoring process
-   **Ping Failures**: Treated as WAN down events, triggering appropriate route changes

The application will continue running indefinitely, logging all errors to stdout/stderr for monitoring and debugging purposes.

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
│   ├── main.rs          # Main application logic, WAN monitoring, and route management
│   └── ssh_connect.rs   # SSH connection handling
└── target/              # Build artifacts (generated)
```

## Key Functions

### `main()`

Main loop that runs continuously:

-   Calls `main_res()` to execute monitoring cycle
-   Handles errors with automatic recovery
-   Waits 5 seconds between monitoring cycles
-   Runs indefinitely until the process is terminated

### `main_res()`

Orchestrates the monitoring process:

-   Establishes SSH connection to MikroTik router
-   Iterates through all configured WAN interfaces
-   Calls `check_and_update_wan()` for each interface
-   Returns errors for handling by the main loop

### `check_and_update_wan()`

Main monitoring function that checks each WAN interface and updates routes accordingly:

-   Pings the destination through the specified VLAN interface
-   Retrieves current route status from MikroTik
-   Enables routes if WAN is up and currently disabled
-   Disables routes if WAN is down and currently enabled
-   Triggers Docker container restart when WAN 2 status changes
-   Logs status changes to console

### `docker_playit_restart()`

Manages Docker container lifecycle during WAN failover events:

-   Executes `docker restart` command for the configured container
-   Logs success or failure of restart operation
-   Called automatically when specific WAN connections change status

### `get_status_route()`

Queries MikroTik router to check if a route is currently enabled or disabled.

### `set_status_route()`

Executes MikroTik commands to enable or disable routes based on WAN status.

### `ping_to_interface()`

Performs ICMP ping tests through a specific network interface to verify connectivity.
