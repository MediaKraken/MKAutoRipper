import RPi.GPIO as GPIO
from time import sleep

# Direction pin from controller
DIR = 26
# Step pin from controller
STEP = 19
# 0/1 used to signify clockwise or counterclockwise.
CW = 1
CCW = 0

# Setup pin layout on PI
GPIO.setmode(GPIO.BOARD)

# Establish Pins in software
GPIO.setup(DIR, GPIO.OUT)
GPIO.setup(STEP, GPIO.OUT)

# Set the first direction you want it to spin
GPIO.output(DIR, CW)

# Run for 200 steps. This will change based on how you set you controller
for x in range(200):
        # Set one coil winding to high
        GPIO.output(STEP,GPIO.HIGH)
        # Allow it to get there.
        sleep(.0005) # Dictates how fast stepper motor will run
        # Set coil winding to low
        GPIO.output(STEP,GPIO.LOW)
        sleep(.0005) # Dictates how fast stepper motor will run
