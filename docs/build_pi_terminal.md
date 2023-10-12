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
libxcursor-dev libxrender-dev libxfixes-dev libpango1.0-dev libgl1-mesa-dev libglu1-mesa-dev

git clone https://github.com/MediaKraken/MKAutoRipper


