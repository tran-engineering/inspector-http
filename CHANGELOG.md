# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/tran-engineering/inspector-http/compare/v0.1.0...v0.1.1) - 2025-10-30

### Other

- remove deployment.md
- scrolling in big payloads

## [0.1.0] - 2025-10-30

### Added
- Initial release
- HTTP server that responds 200 OK to all requests
- Real-time GUI monitoring of incoming HTTP requests
- Two-panel layout: request overview and detailed view
- Display of request method, path, query parameters, headers, and body
- Interactive JSON tree visualization for JSON request bodies
- Copy request body to clipboard
- Save request body to file with native file dialog
- Smart filename generation based on Content-Type header
- Runtime port configuration
- Automatic port detection (finds first available port from 8080+)
- Error recovery with automatic rollback to last working port
- Temporary error message display in status bar
- Color-coded request method badges
- Request body size display
- Performance optimization for large JSON payloads
- Application icon
- Cross-platform support (Linux, macOS, Windows)

### Technical Details
- Built with eframe/EGUI for the GUI
- Hyper for HTTP server functionality
- Tokio for async runtime
- Channel-based communication for port changes
- Graceful server restart on configuration changes
