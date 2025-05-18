# mirai

**mirai** is a lightweight, cross-platform wallpaper manager.

## Features

* **Scheduled Wallpaper Rotation**
  Mirai rotates wallpapers at fixed intervals, calculated from **local midnight**. For example:

  * If `update_interval = 1440`, wallpapers change once per day at midnight.
  * If `update_interval = 60`, they'll change hourly (00:00, 01:00, 02:00, etc.).
  * If it's set to `30`, changes happen every half hour on the clock (00:00, 00:30, 01:00, etc.).

  This deterministic schedule ensures consistency across system reboots.

* Integration with `swww` (Linux) and Windows (experimental)
* TOML-based configuration

## Getting Started

### Prerequisites

**Linux**:

* Wayland compositor
* [`swww`](https://github.com/LGFae/swww) installed and running

**Windows**:

* None, *should* just work

## Installation

### Nix (home manager)

To install `mirai` via Home Manager:

1. Add `mirai` to your Nix flake inputs:

   ```nix
   inputs.mirai.url = "github:iAverages/mirai";
   ```

2. Import the Home Manager module and enable the service:

   ```nix
   {
     imports = [
       inputs.mirai.homeManagerModules.default
     ];

     services.mirai.enable = true;
   }
   ```

3. Then apply your configuration:

   ```bash
   home-manager switch
   ```

This will install and manage `mirai` as a background service that automatically rotates wallpapers according to your configuration.


### Manual Build (All Platforms)

```bash
git clone https://github.com/iAverages/mirai.git
cd mirai
cargo build --release
```

Binary will be in `target/release/mirai`.

## Configuration

Create a file at:

* **Linux**: `~/.config/mirai/mirai.toml`
* **Windows**: `%APPDATA%\\kirsi\\mirai\\config\\mirai.toml`

Local example:

```toml
log_level = "info"
content_manager_type = "local"
update_interval = 1440 # 24 hours in minutes

[local]
# path on disk to your wallpapers folder
# can have nested folders
location = "/home/dan/dotfiles/wallpapers/"
```

Git example:

```toml
log_level = "info"
content_manager_type = "git"
update_interval = 1440 # 24 hours in minutes

[git]
# uri to git server, tested with Github, Gitlab, and Gitea.
# can be a https url, or ssh
# url = "git@github.com:iAverages/dotfiles.git"
url = "https://github.com/iAverages/dotfiles.git"

# path within the git repository to load wallpapers from
path = "wallpapers"
```

## Usage

Simply run the binary:

```bash
./mirai
```

On Windows, use:

```powershell
.\mirai.exe
```

Mirai will start rotating wallpapers based on your settings.

On Windows, you can setup auto start on boot using:

```powershell
.\mirai.exe --autostart true
```

## License

Licensed under the [MIT License](LICENSE).
