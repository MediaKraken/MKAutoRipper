Download https://downloads.raspberrypi.org/raspios_arm64/images/raspios_arm64-2023-05-03/2023-05-03-raspios-bullseye-arm64.img.xz

# make sure the following is updated to the correct sd card path as it will be written over
xzcat ~/Downloads/2023-05-03-raspios-bullseye-arm64.img.xz | sudo dd bs=4M of=/dev/sdb
or 
sudo dd if=~/Downloads/2023-05-03-raspios-bullseye-arm64.img.xz of=/dev/sdb status=progress bs=4M

Boot the pi 3b or higher
do standard PI things

setup ssh server
    sudo raspi-config
    3. Interface options
sudo apt install curl build-essential git

curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh

reload shell

sudo apt-get -y install libx11-dev libxext-dev libxft-dev libxinerama-dev \
libxcursor-dev libxrender-dev libxfixes-dev libpango1.0-dev libgl1-mesa-dev \
libglu1-mesa-dev pkg-config libudev-dev

git clone https://github.com/MediaKraken/MKAutoRipper

------- server vms setup

debian 12.7 for rabbitmq
    netinstall with ssh only
    apt-get install ca-certificates curl
    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
    chmod a+r /etc/apt/keyrings/docker.asc
    echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
    tee /etc/apt/sources.list.d/docker.list > /dev/null
    apt-get update
    apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    docker pull mediakraken/mkrabbitmq:dev
    git clone https://github.com/MediaKraken/MKAutoRipper


# cross compile experiment - mkcode
apt install gcc-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf aarch64-unknown-linux-gnu
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=/usr/bin/arm-linux-gnueabihf-gcc
cargo build --target=armv7-unknown-linux-gnueabihf
cargo build --target=aarch64-unknown-linux-gnu
cross build --target aarch64-unknown-linux-gnu
# https://users.rust-lang.org/t/cross-compiling-arm/96456/3
dpkg --add-architecture aarch64


apt-get install gcc-aarch64-linux-gnu

apt-get install --assume-yes --no-install-recommends \
    libx11-dev:aarch64 libxrandr-dev:aarch64 libasound2-dev:aarch64 \
    libx11-dev:aarch64 libxrandr-dev:aarch64 libxi-dev:aarch64 \
    libgl1-mesa-dev:aarch64 libglu1-mesa-dev:aarch64 \
    libxcursor-dev:aarch64 libxinerama-dev:aarch64


apt-get install --assume-yes --no-install-recommends \
    libx11-dev:arm64 libxrandr-dev:arm64 libasound2-dev:arm64 \
    libx11-dev:arm64 libxrandr-dev:arm64 libxi-dev:arm64 \
    libgl1-mesa-dev:arm64 libglu1-mesa-dev:arm64 \
    libxcursor-dev:arm64 libxinerama-dev:arm64
cross build --target arm64-unknown-linux-gnu

ln -s /usr/lib/gcc-cross/aarch64-linux-gnu/12 /usr/lib/gcc-cross/aarch64-linux-gnu/5
cd /usr/lib/gcc-cross/aarch64-linux-gnu/5/../../../../aarch64-linux-gnu/bin

apt install g++-arm-linux-gnueabihf libc6-dev-armhf-cross
cross build --target=armv7-unknown-linux-gnueabihf
    # this doesn't work as cfltk doesn't have a armv7 version

cross build --target aarch64-unknown-linux-gnu
    # this doesn't work as x11 doesn't have an aarch64 version

# actually since i'm not talking over usb/serial, do I need to do this?

# setup uart ports
sudo nano /boot/config.txt

# add following to bottom
# enable serial interface
enable_uart=1
dtoverlay=uart0
dtoverlay=uart1
dtoverlay=uart2
dtoverlay=uart3
dtoverlay=uart4
dtoverlay=uart5

reboot and verify
ls -al /dev/ttyAMA*

           TXD    PIN  |  RXD      PIN  |  Communication Port
uart0 :  GPIO 14    8  |  GPIO 15   10  |  /dev/ttyAMA0 
uart1 :  GPIO 0    27  |  GPIO 1    28  |  /dev/ttyAMA1
uart2 :  GPIO 4     7  |  GPIO 5    29  |  /dev/ttyAMA2
uart3 :  GPIO 8    24  |  GPIO 9    21  |  /dev/ttyAMA3
uart4 :  GPIO 12   32  |  GPIO 13   33  |  /dev/ttyAMA4
