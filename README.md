# Rust Flutter CLI with TUI (Under Construction)

This Rust CLI tool provides a Text User Interface (TUI) for executing various Flutter commands. It allows users to conveniently interact with Flutter commands using tabs for different categories.

## Features

- Organizes Flutter commands into different tabs for easy navigation.
- Supports commonly used Flutter commands such as `flutter run`, `flutter pub get`, `flutter clean`, etc.
- Provides a seamless command-line experience with a graphical interface.

## Usage

To use the CLI tool, follow these steps:

1. Clone the repository:

   ```bash
   git clone https://github.com/psikosen/rust-flutter-cli.git

## How to Run

cargo run --release

## Tabs and Commands

The CLI tool organizes Flutter commands into different tabs based on their categories:

### Tab 1: Development
- `flutter run`: Run the Flutter application.
- `flutter pub get`: Get dependencies for the Flutter project.
- `flutter channel`: Switch Flutter channels.

### Tab 2: Maintenance
- `flutter clean`: Delete the build/ directory.
- `flutter build`: Build a Flutter application for deployment.
- `flutter doctor`: Check the status of Flutter installation and dependencies.

### Tab 3: Cache Management
- `flutter clean cache`: Delete all cached artifacts.
- `flutter repair`: Repair the Flutter SDK installation.
- `flutter remove cache`: Remove specified artifacts from the cache.

### Tab 4: Device Management
- `flutter devices`: List all connected devices.
- `flutter logs`: Show logs for running Flutter apps.
- `flutter emulators`: List all available emulators.

### Tab 5: Miscellaneous
- `flutter install`: Install a Flutter app on an attached device.
- `flutter pod clean up`: Clean up CocoaPods installation in iOS projects.
- `flutter deintegrate`: Remove Flutter-specific Xcode configuration.
- `flutter repo update`: Update Flutter package repositories.

### Tab 6: Exit
- `exit`: Exit the CLI tool.
