# Newbound Camera

Camera is an app in the [Newbound](https://github.com/mraiser/newbound) appserver ecosystem. To install, add this project to the "data" directory of a Newbound server.

## Installation
These instructions are for Raspberry Pi OS and other flavors of Debian Linux. The Newbound Camera app should work fine on any operating system that supports the Rust compiler, FFMPEG and V4L2.

### Dependencies
    
    sudo apt-get install git libssl-dev pkg-config gcc -y

### Install Rust
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

### Install FFMPEG    

    sudo apt-get update -qq && sudo apt-get -y install \
      autoconf \
      automake \
      build-essential \
      cmake \
      git-core \
      libass-dev \
      libfreetype6-dev \
      libgnutls28-dev \
      libmp3lame-dev \
      libsdl2-dev \
      libtool \
      libva-dev \
      libvdpau-dev \
      libvorbis-dev \
      libxcb1-dev \
      libxcb-shm0-dev \
      libxcb-xfixes0-dev \
      meson \
      ninja-build \
      pkg-config \
      texinfo \
      wget \
      yasm \
      zlib1g-dev \
      libunistring-dev \
      libaom-dev \
      libdav1d-dev \
      nasm \
      libx264-dev \
      libx265-dev \
      libnuma-dev \
      libvpx-dev \
      libfdk-aac-dev \
      libopus-dev \
      libdav1d-dev 
      
    mkdir -p ~/ffmpeg_sources ~/bin
    cd ~/ffmpeg_sources && \
    wget -O ffmpeg-snapshot.tar.bz2 https://ffmpeg.org/releases/ffmpeg-snapshot.tar.bz2 && \
    tar xjvf ffmpeg-snapshot.tar.bz2 && \
    cd ffmpeg && \
    PATH="$HOME/bin:$PATH" PKG_CONFIG_PATH="$HOME/ffmpeg_build/lib/pkgconfig" ./configure \
      --prefix="$HOME/ffmpeg_build" \
      --pkg-config-flags="--static" \
      --extra-cflags="-I$HOME/ffmpeg_build/include" \
      --extra-ldflags="-L$HOME/ffmpeg_build/lib" \
      --extra-libs="-lpthread -lm" \
      --ld="g++" \
      --bindir="$HOME/bin" \
      --enable-gpl \
      --enable-gnutls \
      --enable-libaom \
      --enable-libass \
      --enable-libfdk-aac \
      --enable-libfreetype \
      --enable-libmp3lame \
      --enable-libopus \
      --enable-libdav1d \
      --enable-libvorbis \
      --enable-libvpx \
      --enable-libx264 \
      --enable-libx265 \
      --enable-nonfree && \
    PATH="$HOME/bin:$PATH" make && \
    make install && \
    hash -r

    cd ~
    sudo mv bin/ff* /usr/bin/
    rm -rf ffmpeg_build
    rm -rf ffmpeg_sources

### Install Newbound

    git clone https://github.com/mraiser/newbound.git Newbound
    cd Newbound
    cargo build --release --features=serde_support

### Install Newbound Camera

    mkdir github
    cd github
    git clone https://github.com/mraiser/camera.git camera
    cd ../data
    ln -s ../github/camera/data/camera camera
    cd ../runtime
    ln -s ../github/camera/runtime/camera camera
    cd ../

### Startup Newbound

    # Launch Newbound as an app
    # This will automatically open a logged-in browser window
    # Use feature "headless" to suppress browser
    # Use feature "webview" to launch as standalone app instead of browser 
    cargo run --release --features=serde_support

OR

    # Install and launch Newbound as a service
    printf "[Unit]
    Description=Newbound

    [Service]
    User=$USER
    Group=$USER
    WorkingDirectory=/home/$USER/Newbound
    ExecStart=bash -c '/home/$USER/.cargo/bin/cargo run --release --features=serde_support &'
    Type=forking
    RemainAfterExit=yes

    [Install]
    WantedBy=multi-user.target
    " | sudo tee /etc/systemd/system/newbound.service
    
    sudo systemctl daemon-reload
    sudo systemctl enable newbound
    sudo systemctl start newbound

### Activate Camera app
When Newbound launches for the first time, it will launch the "Applications" app in a web browser with  inactive apps hidden. Click the toggle switch in "Applications" to show the Camera app, select it, then click the "ACTIVATE" button.

# Useful config files 
(after first run)

    # Show HTTP port
    cat config.properties

    # Show admin user password
    cat users/admin.properties

# Sample Camera config file 
(created automatically when saving preferences in app)

    # Newbound/runtime/camera/botd.properties
    dvr=true
    motion=true
    format=SBGGR12_CSI2P
    rotation=0
    framerate=30
    motion_noise_cancel=4
    video_encoder=copy
    days=2.0
    camera=Raspberry Pi - imx477 [4056x3040] (/base/soc/i2c0mux/i2c@1/imx477@1a)
    device=libcamera-apps
    resolution=1920x1080
    motion_sensitivity=8
    storage=/mnt/usb/storage

