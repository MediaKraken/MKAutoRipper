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
