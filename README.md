# Newbound Camera

Camera is an app in the [Newbound](https://github.com/mraiser/newbound) appserver ecosystem: live view, DVR with
time-travel playback and clip download, motion detection with event browsing, and peer-to-peer remote viewing —
all served from your own hardware. The live view plays a fragmented-MP4 stream in any modern browser (MSE),
falling back to still-image polling automatically.

## Installation

These instructions are for Raspberry Pi OS and other flavors of Debian Linux (including Ubuntu). The app works on
any OS with the Rust compiler, FFmpeg, and either a CSI camera stack (rpicam/libcamera) or V4L2 devices.

### Dependencies

    sudo apt-get update
    sudo apt-get install -y git build-essential libssl-dev pkg-config curl ffmpeg v4l-utils

The distribution FFmpeg is all that is needed — no source build. On Raspberry Pi OS the CSI camera tools ship
preinstalled (`rpicam-vid`/`rpicam-still` on Bookworm and later; the older `libcamera-*` names are also detected).
USB webcams need only `v4l-utils`.

### Install Rust

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

### Install Newbound

    git clone https://github.com/mraiser/newbound.git Newbound
    cd Newbound
    cargo build --release --features=serde_support

### Install the Camera app

**From a running Newbound** (easiest): use the dev library's GitHub importer with this repository's URL
(`https://github.com/mraiser/camera.git`). It clones the repo whole into `repositories/camera`, links the app into
place, and builds it; restart Newbound once when it says so.

**Manually** (from the `Newbound` directory):

    git clone https://github.com/mraiser/camera.git repositories/camera
    ln -s "$PWD/repositories/camera/data/camera" data/camera
    ln -s "$PWD/repositories/camera/runtime/camera" runtime/camera
    ln -s "$PWD/repositories/camera/camera" camera
    ./target/release/newbound rebuild
    cargo build --release --features=serde_support
    (cd camera && cargo build --release --features=serde_support)

The camera library is a hot-reload crate: after the first build, changes to its Rust recompile and reload into a
running server with no restart.

### Start Newbound

    # Launch directly — opens a logged-in browser window on first run
    ./target/release/newbound

Or install it as a service:

    printf "[Unit]
    Description=Newbound
    After=network.target

    [Service]
    User=$USER
    WorkingDirectory=/home/$USER/Newbound
    ExecStart=/home/$USER/Newbound/target/release/newbound
    Restart=on-failure

    [Install]
    WantedBy=multi-user.target
    " | sudo tee /etc/systemd/system/newbound.service

    sudo systemctl daemon-reload
    sudo systemctl enable --now newbound

### NixOS

Newbound's checkout carries a `default.nix` dev shell with the build toolchain. The camera app's runtime tools are
its own concern — bring them into the environment alongside it:

    cd Newbound
    nix-shell default.nix -p ffmpeg v4l-utils    # add rpicam-apps for Pi CSI cameras
    cargo build --release --features=serde_support
    ./target/release/newbound

(A flake/NixOS module is not provided yet.)

### Activate the Camera app

When Newbound launches for the first time it opens the "Applications" app with inactive apps hidden. Click the
toggle switch to show the Camera app, select it, then click **ACTIVATE**.

## Useful config files

(after first run)

    # Show HTTP port
    cat config.properties

    # Show admin user password
    cat users/admin.properties

## Sample Camera config file

(created automatically when saving preferences in the app)

    # Newbound/runtime/camera/botd.properties
    dvr=true
    motion=true
    format=SBGGR12_CSI2P
    rotation=0
    framerate=30
    motion_noise_cancel=4
    video_encoder=copy
    days=2.0
    camera=Raspberry Pi - imx477 [4056x3040 10-bit RGGB] (/base/soc/i2c0mux/i2c@1/imx477@1a)
    device=libcamera:0
    resolution=1920x1080
    motion_sensitivity=8
    storage=/mnt/usb/storage

`device` values: `libcamera:<index>` selects a CSI camera by index (configs saved by older versions with
`device=libcamera-apps` still work and mean index 0); a `/dev/video*` path selects a V4L2 device. Point `storage`
at a disk with room for the DVR archive — retention is `days` (default 2).
