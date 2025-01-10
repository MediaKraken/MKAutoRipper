import os
import shlex
import subprocess
import sys
import time

for drive_number in range(0, 2): # set one higher than number of drives!
    print(drive_number)
    subprocess.Popen(['eject', '/dev/sr%s' % drive_number])
    time.sleep(10)
    subprocess.Popen(['eject', '-t', '/dev/sr%s' % drive_number])
