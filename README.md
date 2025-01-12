# MKAutoRipper
Code/models/etc for Media Auto Ripper hardware

debian on main box for ease of makemkv and eject commands

raspberry pi will run the show telling server when cd's should load

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

must sudo run the pi_terminal app or you don't have access to gpio pins