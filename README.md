# 🪟 Windows Transparency Wizard 🧙‍♂️

Hey there, cool cats! Welcome to the Windows Transparency Wizard, a groovy little Rust application that lets you jazz up your Windows experience by tweaking the transparency of your open windows. Now with a slick GUI and system tray support!

## 🌟 Features

- Sleek graphical interface to manage window transparency
- System tray integration - keep it running in the background
- Live window transparency preview and adjustment
- Persistent settings for your favorite windows
- Global default opacity setting for all windows
- Real-time window list refresh
- Smart window detection by title or executable name

## 🚀 Getting Started

1. Clone this rad repository
2. Make sure you've got Rust installed (it's the bee's knees)
3. Copy `config.yaml.example` to `config.yaml` and customize it to your heart's content
4. Run `cargo build` to get everything set up
5. Launch the app with `cargo run` and watch the magic happen!

## 🎛️ Configuration

The `config.yaml` file is where the real party happens. Here's how to set it up:

```yaml
default_opacity: 90  # Optional: Sets a groovy baseline for all windows

specific_windows:
  - title: "Notepad"
    opacity: 80
  - executable: "chrome.exe"
    opacity: 95
  - title: "Visual Studio Code"
    executable: "Code.exe"
    opacity: 85
```

* Toggle default opacity right from the GUI
* Save window-specific settings with a single click
* Settings persist automatically when you make changes
* Mix and match window titles and executables
* Set opacity from 0-100% using the slider

## 🛠️ How It Works

This wizard uses some powerful Rust incantations powered by:

* `iced`: For that sweet, sweet GUI goodness
* `windows` crate: Speaking the secret language of your OS
* `serde`: Handling configuration magic
* System tray integration for background vibes

The main spell components are:

* `main.rs`: The heart of the operation with the new GUI implementation
* `config.rs`: Handles loading, saving, and parsing of your groovy configuration
* `config.yaml`: Your personal spellbook for customizing window transparency

## 🎭 Contributing
Feel like adding your own flavor to this funky mix? Contributions are always welcome! Just keep it cool, follow the code style, and make sure your changes are as smooth as jazz.

## 📜 License
This project is licensed under the MIT License - see the LICENSE file for details. Spread the love, share the code!

Now go forth and make your Windows experience as transparent as you want it to be! ✨