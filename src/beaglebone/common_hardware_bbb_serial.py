# send serial signals "over the wire" via beaglebone

# Serial2 TX = pin 21 on P9 header
# Serial2 RX = pin 22 on P9 header

from bbio import *


def com_hardware_bbb_serial_setup(selected_speed):
    """
    Set connection speed for connection
    """
    baud_rates = [300, 1200, 2400, 9600, 19200,
                  38400, 57600, 115200, 230400, 460800]
    # Start Serial2 at selected baud:
    Serial2.begin(baud_rates[selected_speed])


def com_hardware_bbb_data_check():
    """
    Check and read info on serial port
    """
    if Serial2.available():
        # There's incoming data
        data = ''
        while Serial2.available():
            # If multiple characters are being sent we want to catch
            # them all, so add received byte to our data string and
            # delay a little to give the next byte time to arrive:
            data += Serial2.read()
            delay(5)
        print(("Data received:\n  '%s'" % data))


def com_hardware_bbb_data_write(data_string):
    """
    Write data to connected device
    """
    print(("Data write:\n  '%s'" % data_string))
    Serial2.write(data_string)
