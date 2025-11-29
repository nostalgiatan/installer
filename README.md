# SeeSea Self-Contained Installer

A powerful, cross-platform self-contained installer built with Rust, supporting Windows, Linux, and macOS.

## 📋 Features

- **🔄 Cross-platform**: Supports Windows, Linux, and macOS
- **⚡ High Performance**: Built with Rust for speed and reliability
- **📦 Efficient Packaging**: Uses zstd compression for small package sizes
- **🎨 User-friendly**: Provides both command-line and graphical interfaces
- **🔧 Customizable**: Supports custom installation commands and options
- **🔒 Secure**: No external dependencies, self-contained executable
- **📝 Well-documented**: Comprehensive API and usage documentation

## 🚀 Getting Started

### Prerequisites

- Rust 1.88.0 or higher
- Cargo package manager

### Installation

#### Build from Source

```bash
# Clone the repository
git clone git@github.com:nostalgiatan/installer.git
cd installer

# Build the installer
cargo build --release

# Run the installer
./target/release/seesea-installer
```

#### Download Pre-built Binaries

Pre-built binaries are available for all supported platforms on the [GitHub Releases](https://github.com/nostalgiatan/installer/releases) page.

## 📖 Usage

### Command Line Options

```bash
# Show help
seesea-installer --help

# Custom installation directory
seesea-installer --install-dir /opt/seesea

# Silent installation (no interaction)
seesea-installer --quiet

# Custom configuration file
seesea-installer --config custom-installer.toml

# Run specific command
seesea-installer install
seesea-installer uninstall
seesea-installer repair
```

### Configuration File

The installer uses a TOML configuration file with the following structure:

```toml
[project]
name = "SeeSea"
version = "1.0.0"
description = "Privacy-focused metasearch engine"
author = "SeeSea Team"

[install_options]
default_dir = "/opt/seesea"
create_desktop_shortcut = true
create_start_menu_shortcut = true
add_to_path = true
create_uninstaller = true

[commands]
[[commands]]
name = "start-service"
program = "/opt/seesea/bin/seesea-service"
args = ["start"]
working_dir = "/opt/seesea"
background = true

[platform.windows]
default_dir = "C:\\Program Files\\SeeSea"

[platform.linux]
default_dir = "/usr/local/seesea"

[platform.macos]
default_dir = "/Applications/SeeSea"
```

## 📁 Project Structure

```
installer/
├── src/
│   ├── cli.rs          # Command-line argument parsing
│   ├── config.rs       # Configuration management
│   ├── installer.rs    # Core installation logic
│   ├── packager.rs     # zstd compression/decompression
│   ├── platform/       # Platform-specific code
│   │   ├── linux.rs
│   │   ├── macos.rs
│   │   ├── mod.rs
│   │   └── windows.rs
│   └── utils.rs        # Utility functions
├── tests/              # Integration tests
├── Cargo.toml          # Rust dependencies
└── README.md           # This file
```

## 🔧 API Usage

The installer can also be used as a library in other Rust projects:

```rust
use seesea_installer::{Config, Installer, load_config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config("installer.toml")?;
    
    // Create installer instance
    let mut installer = Installer::new(config, &Default::default())?;
    
    // Run installation
    installer.install()?;
    
    Ok(())
}
```

## 🎯 Supported Platforms

| Platform | Architecture | Status |
|----------|--------------|--------|
| Windows  | x64          | ✅ Supported |
| Linux    | x64          | ✅ Supported |
| macOS    | x64          | ✅ Supported |
| macOS    | ARM64        | ✅ Supported |

## 🔄 CI/CD

The project uses GitHub Actions for continuous integration and deployment:

- **Build**: Multi-platform builds for Windows, Linux, and macOS
- **Test**: Automated testing for all components
- **Release**: Automatic release creation on tag push

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📞 Support

For issues and feature requests, please open an issue on [GitHub Issues](https://github.com/nostalgiatan/installer/issues).

## 📌 Roadmap

- [ ] Add more platform-specific features
- [ ] Improve graphical installation interface
- [ ] Add support for more compression algorithms
- [ ] Implement plugin system for custom installers
- [ ] Add digital signature support

## 📊 Performance

- **Compression Ratio**: Up to 90% reduction in file size
- **Installation Speed**: Fast installation with minimal overhead
- **Memory Usage**: Low memory footprint
- **Disk Usage**: Small executable size

## 🔒 Security

- No external dependencies
- Static linking for all platforms
- Secure file permissions
- Proper error handling
- Comprehensive logging

---

Built with ❤️ using Rust
