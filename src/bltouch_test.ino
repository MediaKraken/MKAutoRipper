#include <Servo.h>

Servo myservo;

const byte interruptPin = 2;   // White probe output wire
const int servoPin = 3;        // BL-Touch control pin
const int linkPin = A1;        // COOLANT_ENABLE input
const int ledPin = LED_BUILTIN;

const int PROBE_DOWN_ANGLE = 10;
const int PROBE_UP_ANGLE = 90;

const unsigned long SERVO_SETTLE_MS = 15;
const unsigned long RETRACT_HOLD_MS = 200;
const unsigned long PROBE_TIMEOUT_MS = 2000;
const unsigned long IDLE_POLL_MS = 25;

volatile bool probeTriggered = false;

enum ProbeState {
  STATE_IDLE,
  STATE_DEPLOYING,
  STATE_WAITING_FOR_TRIGGER,
  STATE_RETRACTING
};

ProbeState probeState = STATE_IDLE;
unsigned long stateStartedAt = 0;
unsigned long lastIdlePollAt = 0;

void blink() {
  probeTriggered = true;
}

bool probeEnabled() {
  int rawValue = analogRead(linkPin);
  int mappedValue = map(rawValue, 0, 1023, 0, 1);
  return mappedValue == 0;
}

void setState(ProbeState newState) {
  probeState = newState;
  stateStartedAt = millis();
}

void deployProbe() {
  myservo.write(PROBE_DOWN_ANGLE);
}

void retractProbe() {
  myservo.write(PROBE_UP_ANGLE);
}

void setup() {
  myservo.attach(servoPin);

  pinMode(ledPin, OUTPUT);
  pinMode(linkPin, INPUT_PULLUP);
  pinMode(interruptPin, INPUT_PULLUP);

  attachInterrupt(digitalPinToInterrupt(interruptPin), blink, RISING);

  retractProbe();
  digitalWrite(ledPin, LOW);
  setState(STATE_IDLE);
}

void loop() {
  unsigned long now = millis();

  switch (probeState) {
    case STATE_IDLE:
      if (now - lastIdlePollAt >= IDLE_POLL_MS) {
        lastIdlePollAt = now;

        if (probeEnabled()) {
          probeTriggered = false;
          digitalWrite(ledPin, HIGH);
          deployProbe();
          setState(STATE_DEPLOYING);
        } else {
          retractProbe();
          digitalWrite(ledPin, LOW);
        }
      }
      break;

    case STATE_DEPLOYING:
      if (!probeEnabled()) {
        retractProbe();
        digitalWrite(ledPin, LOW);
        setState(STATE_IDLE);
        break;
      }

      if (now - stateStartedAt >= SERVO_SETTLE_MS) {
        setState(STATE_WAITING_FOR_TRIGGER);
      }
      break;

    case STATE_WAITING_FOR_TRIGGER:
      if (!probeEnabled()) {
        retractProbe();
        digitalWrite(ledPin, LOW);
        setState(STATE_IDLE);
        break;
      }

      if (probeTriggered || (now - stateStartedAt >= PROBE_TIMEOUT_MS)) {
        retractProbe();
        setState(STATE_RETRACTING);
      }
      break;

    case STATE_RETRACTING:
      if (now - stateStartedAt >= RETRACT_HOLD_MS) {
        probeTriggered = false;
        digitalWrite(ledPin, LOW);
        setState(STATE_IDLE);
      }
      break;
  }
}