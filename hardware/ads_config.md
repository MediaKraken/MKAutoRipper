# config the pi for i2c
sudo raspi-config
interface options, advanced, i2c
sudo apt-get install -y i2c-tools
sudo reboot
sudo i2cdetect -y 1
verify the devices shows up

# https://github.com/CCSnell/BLTouch4bCNC
# https://github.com/chepo92/3DtouchTestSimple
# https://github.com/BNieveld/CrTouch-Tester

# to test - add to
/etc/udev/rules.d/99-arduino.rules
SUBSYSTEMS=="usb", ATTRS{idVendor}=="2341", GROUP="plugdev", MODE="0666"

# pinouts
ADS1x15 VDD to Raspberry Pi 3.3V                PIN 1
ADS1x15 GND to Raspberry Pi GND                 PIN 9, 25, 39
ADS1x15 SCL to Raspberry Pi SCL     GPIO 3      PIN 5
ADS1x15 SDA to Raspberry Pi SDA     GPIO 2      PIN 3
-32768 to 32767 is the probable range

# cr touch
COLOUR 	PIN
White 	GND
Black 	+5V
Yellow 	SIG
Red 	GND
Blue 	OUT 

COLOR (in photo below) 	PIN Description
RED 	GND
GREEN 	+5V
L. BLUE 	PWM Signal
D. BLUE 	GND (for probe/endstop)
PURPLE 	SIGNAL (for probe/endstop) 

# bltouch wiring
https://www.ebay.com/itm/294082430266?_skw=bl+touch&itmmeta=01JSN98GSG5SY1FY9HFG8Y2D22&hash=item4478adc53a:g:cZoAAOSwBUFnEjai&itmprp=enc%3AAQAKAAAA4FkggFvd1GGDu0w3yXCmi1c2XHPyGxmUr%2BEdb%2B%2BnjALQvPmYWDjna84YDzrcwfUY7ab7aNnyemgSsXRlZwTnxZrQiDWE0zdxRXfehliGDRD8x6rvyutpd773n06JelWSf%2FjHGAhVfqhI0lTBd%2BO%2F8CdY9XR3%2FToEF4v8iINr0%2BkbX2DRbXIDjvCLkWfDz9wvEYtCHPzB0fszKWJENvwwJFzTt9V4MEl%2FRYACreepCMQDQYOdNxqotK7r%2FcBq5zdrzjRa9ZayAVfAutyAq723U2hYT0f1o9KNmOwYeK1aPxxo%7Ctkp%3ABFBMjI2iqc1l