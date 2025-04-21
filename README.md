# MKAutoRipper
Code/models/etc for Media Auto Ripper hardware

debian on main box for ease of makemkv and eject commands

raspberry pi will run the show telling server when cd's should load

sudo raspi-config
interface options, advanced, i2c
sudo apt-get install -y i2c-tools
sudo reboot
sudo i2cdetect -y 1
verify the devices shows up

I2C (SDA)	GPIO 2
I2C (SCL)	GPIO 3

ADS1x15 VDD to Raspberry Pi 3.3V
ADS1x15 GND to Raspberry Pi GND
ADS1x15 SCL to Raspberry Pi SCL
ADS1x15 SDA to Raspberry Pi SDA

**********************************

Horizontal stepper - moves entire arm assembly on tracks left/right
vertical stepper - moves arm assembly on track up/down
photo stepper = moves platform back and forth for photos of media

relay - vacuum pump to hold disc
relay - led for disk image photo

********************

setup the raspbian image for the pi

dd bs=4M if=2017-11-29-raspbian-stretch.img of=/dev/sdb conv=fsync

***************************************


GPIOs up to 8: default state is 1 (HIGH, or close to 3.3V).
GPIOs 9 to 27: default state is 0 (LOW, or close to 0V).

The Raspberry Pi supports 2 hardware based PWM channels. You can access these two channels via 2 separate sets of 4 GPIO header pins, but still limited to only 2 channels (2 unique PWM timing configurations).
    Hardware PWM available on GPIO12, GPIO13, GPIO18, GPIO19