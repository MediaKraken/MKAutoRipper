# general pin input/output for motion detector device
import time

import Adafruit_BBIO.GPIO as GPIO

GPIO.setup('P9_15', GPIO.IN)

file_handle = open('movement_log.txt', 'w')

while True:
    GPIO.wait_for_edge("P9_15", GPIO.RISING)
    log_start = time.strftime("%a, %d %b %Y %H:%M:%S")
    GPIO.wait_for_edge("P9_15", GPIO.FALLING)
    log_end = time.strftime("%a, %d %b %Y %H:%M:%S")
    file_handle.write("+" + "-" * 40 + "\n")
    file_handle.write("| Start: %s\n" % log_start)
    file_handle.write("| End: %s\n" % log_end)
