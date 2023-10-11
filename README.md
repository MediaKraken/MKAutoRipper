# MKAutoRipper
Code/models/etc for Media Auto Ripper hardware

debian on main box for ease of makemkv and eject commands

server listens via twisted

raspberry pi will run the show telling server when cd's should load

will also tell the ardionos to do their jobs via usb hub

just crossover cable/adapter

**********************************

Horizontal stepper - moves entire arm assembly on tracks left/right
vertical stepper - moves arm assembly on track up/down

spinner stepper - rotates cd for buffing

buffer stepper - rotates buffing wheel

relay - vacuum pump to hold disc
relay - water pump to cool/wash disc

********************

setup the raspbian image for the pi

dd bs=4M if=2017-11-29-raspbian-stretch.img of=/dev/sdb conv=fsync

***************************************

A CD is 4.72 inches (120 millimeters) in diameter and .047 inches (1.2 millimeters) thick.
The positioning hole in the middle is .59 of an inch (15 millimeters) in diameter.

So, have a carriage bolt through the wood which couples to nema32


So, wood circle 5" in diameter. 3/4" thick hardwood. Clear-coated.
Tool grip stuff to hold cd in place.


1pcs NEMA23 570OZ/IN 5A 3/8� DUAL SHAFT STEPPER MOTOR
