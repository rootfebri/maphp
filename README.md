# maphp 🐘

A fast and efficient PHP version manager written in Rust. maphp allows you to easily install, manage, and switch between different PHP versions on your system.

## Features

- 🚀 **Fast**: Built with Rust for optimal performance
- 📦 **Easy Installation**: Download and compile PHP versions automatically [Read [PHP Dependency](https://github.com/php/php-src/?tab=readme-ov-file#building-php-source-code) required for your system to compile PHP]
- 🔄 **Version Switching**: Seamlessly switch between installed PHP versions
- 🗂️ **Clean Management**: Organized storage of PHP installations
- 🎯 **Interactive CLI**: User-friendly command-line interface with fuzzy selection
- 🔧 **Automatic Configuration**: Sets up PHP configurations automatically

## Installation

### Prerequisites

- Rust (latest stable version)
- [Build tools](https://github.com/php/php-src/?tab=readme-ov-file#building-php-source-code) for compiling PHP from source

### Build from Source

```bash
git clone https://github.com/rootfebri/maphp.git
cd maphp
cargo build --release
```

The binary will be available at `target/release/maphp`.

### Add to PATH

Add the maphp binary to your PATH:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="/path/to/maphp/target/release:$PATH"
```

## Usage

### Install a PHP Version

```bash
# Install PHP 8.3.0
maphp install 8.3.0

# Install with php- prefix (automatically stripped)
maphp install php-8.2.15
```

### List Available/Installed Versions

```bash
# List all available PHP versions
maphp list

# List only installed versions
maphp list --installed
```

### Switch PHP Version

```bash
# Switch to a specific version
maphp use 8.3.0

# Interactive selection from installed versions
maphp use
```

### Remove a PHP Version

```bash
# Remove a specific version
maphp remove 8.2.15

# Interactive selection for removal
maphp remove
```

### Configuration

maphp stores all PHP installations in `~/.maphp/` by default. You can customize this location:

```bash
# Use custom directory
maphp --work-dir /custom/path install 8.3.0
```

### Environment Variables

- `HOME`: Default work directory (can be overridden with `--work-dir`)

## Directory Structure

```
~/.maphp/
├── archives/           # Downloaded and extracted PHP sources
│   ├── 8.3.0/
│   ├── 8.2.15/
│   └── ...
├── bin/               # Symlinks to active PHP version
└── tags.json      # Cached git tags information
```

## Commands

| Command             | Description             | Example                  |
|---------------------|-------------------------|--------------------------|
| `install <version>` | Install a PHP version   | `maphp install 8.3.0`    |
| `remove [version]`  | Remove a PHP version    | `maphp remove 8.2.15`    |
| `use [version]`     | Switch to a PHP version | `maphp use 8.3.0`        |
| `list`              | List PHP versions       | `maphp list --installed` |

## Options

| Option              | Short | Description               |
|---------------------|-------|---------------------------|
| `--work-dir <PATH>` | `-w`  | Set custom work directory |
| `--help`            | `-h`  | Show help information     |
| `--version`         | `-V`  | Show version information  |

## Examples

```bash
# Install the latest PHP 8.3
maphp install 8.3.0

# List all installed versions
maphp list --installed

# Switch to PHP 8.2 interactively
maphp use

# Remove old PHP version
maphp remove 7.4.33

# Install with custom work directory
maphp --work-dir ~/my-php install 8.3.0
```

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo clippy
cargo fmt
```

## Dependencies

- **anyhow**: Error handling
- **reqwest**: HTTP client for downloads
- **clap**: Command-line argument parsing
- **tokio**: Async runtime
- **dialoguer**: Interactive prompts
- **indicatif**: Progress bars
- **serde**: Serialization
- **tar**: Archive extraction

## Platform Support

- ✅ Linux
- ✅ macOS (likely)
- ❌ Windows (planned for future release)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Node Version Manager (nvm) and similar tools
- Built with the amazing Rust ecosystem
- Thanks to the PHP community for maintaining excellent releases

## Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/rootfebri/maphp/issues) page
2. Create a new issue with detailed information
3. Include your system information and maphp version

---

Made with ❤️ and 🦀 Rust
