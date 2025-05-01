/*
  BLTouch Test sensor program
  Connect 5V to Red
  Connect Gnd to Brown
  Connect Pin 9 to Orange / Yellow
  Connect Pin 2 to White.

  CRTouch Test sensor program
  Connect Gnd to White
  Connect 5V to Black
  Connect Pin 9 to Yellow
  Connect Gnd to Red
  Connect Pin 2 to Blue

  Serial port to 9600 and dont send CR or LF

  Send 1 to Pin Down
  Send 2 to Pin Up
  Send 3 to Test
  Send 4 to SW Reset
  Send 5 to Reset

*/

#include <Servo.h>

Servo myservo;  

int val;  
int incomingByte = 0;

const byte BLTouchPin = 2;        // Connect the white wire from the BLTouch to this pin / crtouch blue
const byte BLTouchControl = 9;    // Connect the orange wire from the BLTouch sensor to this pin / crtouch yellow
volatile byte state = LOW;

void setup() {
  Serial.begin(9600);
  myservo.attach(BLTouchControl); 
  pinMode(BLTouchPin, INPUT_PULLUP);
  attachInterrupt(digitalPinToInterrupt(BLTouchPin), touch, CHANGE);
  myservo.write(60);
}

void loop() {
    // send data only when you receive data:
    if (Serial.available() > 0) {
            // read the incoming byte:
            incomingByte = Serial.read();
            switch (incomingByte) {
              case 49: // 1
                Serial.println("Push pin down");
                myservo.write(10);
                break;
              case 50: // 2
                Serial.println("Pull pin up");
                myservo.write(90);
                break;
              case 51: // 3
                Serial.println("Self Test");
                myservo.write(120);
                break;
              case 52: // 4
                Serial.println("Touch SW Mode (M119)");
                myservo.write(60);
                break;
              case 53: // 5
                Serial.println("Alarm Release & Reset");
                myservo.write(160);
                break;
            }
    }
}

void touch() {
  Serial.println("Touch");
  state = !state;
}
