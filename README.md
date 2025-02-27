# Pi Remote Manager

A Rust-based application for managing files and images on remote Raspberry Pi devices.

## Features

- Connect to remote Raspberry Pi devices via SSH
- Browse and manage files on remote systems
- Preview various file types including documents, images, and text files
- Process and manipulate images remotely
- Transfer files using SCP or rsync
- Intuitive user interface for all operations

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (version 1.70.0 or later)
- Cargo (comes with Rust)
- Git
- OpenSSL development libraries

## Installation

### 1. Install Rust

If you don't have Rust installed, you can install it using rustup:

```bash
# For Unix-like operating systems (Linux, macOS)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# For Windows, download and run rustup-init.exe from:
# https://rustup.rs/
```

Follow the on-screen instructions to complete the installation. After installation, you may need to restart your terminal or run:

```bash
source $HOME/.cargo/env
```

### 2. Clone the Repository

```bash
git clone https://github.com/cesco345/pi_remote_manager.git
cd pi_remote_manager
```

### 3. Build the Project

```bash
cargo build --release
```

The executable will be located at `target/release/pi_remote_manager`.

## Usage

### Running the Application

```bash
./target/release/pi_remote_manager
```

Or on Windows:

```bash
.\target\release\pi_remote_manager.exe
```

### Configuration

The application uses a configuration file located at `~/.config/pi_remote_manager/config.toml` (Unix-like systems) or `%APPDATA%\pi_remote_manager\config.toml` (Windows).

Create this file with the following template:

```toml
[general]
default_remote_dir = "/home/pi"
default_local_dir = "/path/to/your/local/directory"

[connections]
[[connections.saved]]
name = "My Pi"
host = "raspberrypi.local"
port = 22
username = "pi"
# Either password or key_path should be specified
password = ""
key_path = "~/.ssh/id_rsa"
```

## Project Structure

```
./src
├── config                 # Application configuration
│   ├── app_config.rs
│   └── mod.rs
├── core                   # Core functionality
│   ├── file               # File handling
│   │   ├── file_type.rs
│   │   ├── mod.rs
│   │   └── preview.rs
│   ├── image              # Image processing
│   │   ├── mod.rs
│   │   ├── operations.rs
│   │   └── processor.rs
│   ├── mod.rs
│   └── utils              # Utility functions
│       ├── error.rs
│       ├── image_utils.rs
│       └── mod.rs
├── main.rs                # Application entry point
├── transfer               # File transfer methods
│   ├── method.rs
│   ├── mod.rs
│   ├── rsync.rs
│   ├── ssh.rs
│   └── transfer_method.rs
└── ui                     # User interface
    ├── browser            # File browsing
    │   ├── file_browser.rs
    │   ├── mod.rs
    │   └── remote_browser.rs
    ├── dialogs.rs
    ├── file_browser.rs
    ├── hybrid_main_window.rs
    ├── image_view.rs
    ├── main_window.rs
    ├── main_window_adapter.rs
    ├── mod.rs
    ├── operations_panel.rs
    ├── preview            # File previews
    │   ├── document_preview.rs
    │   ├── image_preview.rs
    │   ├── mod.rs
    │   ├── preview.rs
    │   ├── preview_panel.rs
    │   └── text_preview.rs
    └── transfer_panel.rs
```

## Development Setup

### Required Dependencies

#### Ubuntu/Debian:

```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev libgtk-3-dev
```

#### Fedora:

```bash
sudo dnf install gcc pkg-config openssl-devel gtk3-devel
```

#### macOS:

```bash
brew install openssl pkg-config gtk+3
```

#### Windows:

- Install the [MSVC build tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [GTK3 for Windows](https://www.gtk.org/docs/installations/windows/)

### Setting Up Development Environment

1. Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone the repository:

```bash
git clone https://github.com/yourusername/pi_remote_manager.git
cd pi_remote_manager
```

3. Set up your `Cargo.toml` dependencies:

```toml
[dependencies]
# SSH and file transfer
ssh2 = "0.9"
russh = "0.40"
rsync = "0.3"

# UI
gtk = "0.18"
gio = "0.18"

# Image processing
image = "0.24"
imageproc = "0.23"

# Utilities
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
log = "0.4"
env_logger = "0.10"
```

4. Build in development mode:

```bash
cargo build
```

5. Run tests:

```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Rust](https://www.rust-lang.org/)
- [GTK-rs](https://gtk-rs.org/)
- [SSH2 crate](https://docs.rs/ssh2/latest/ssh2/)
- [image crate](https://docs.rs/image/latest/image/)
