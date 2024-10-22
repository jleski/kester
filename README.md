# 🪟 Windows Transparency Wizard 🧙‍♂️

Hey there, cool cats! Welcome to the Windows Transparency Wizard, a groovy little Rust application that lets you jazz up your Windows experience by tweaking the transparency of your open windows. It's like giving your desktop a pair of funky sunglasses!

## 🌟 Features

- Lists all visible windows with their titles, classes, and more
- Grabs the executable name for each window (how's that for detective work?)
- Checks out the current transparency level of each window
- Sets custom transparency levels based on your configuration
- Keeps it cool by not messing with windows you don't specify

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

* If you set `default_opacity`, all windows get that funky tint
* Add windows to `specific_windows` to give them their own unique vibe
* You can specify windows by `title`, `executable`, or `both`
* Set `opacity` to 100 to keep a window fully opaque (no sunglasses needed!)

## 🛠️ How It Works

This wizard uses some powerful Rust incantations (powered by the `windows` crate) to communicate with the Windows API. It's like speaking the secret language of your operating system!

The main spell components are:

* `main.rs`: The heart of the operation, enumerating windows and setting transparency
* `config.rs`: Handles the loading and parsing of your groovy configuration
* `config.yaml`: Your personal spellbook for customizing window transparency

## 🎭 Contributing
Feel like adding your own flavor to this funky mix? Contributions are always welcome! Just keep it cool, follow the code style, and make sure your changes are as smooth as jazz.

## 📜 License
This project is licensed under the MIT License - see the LICENSE file for details. Spread the love, share the code!

Now go forth and make your Windows experience as transparent as you want it to be! ✨