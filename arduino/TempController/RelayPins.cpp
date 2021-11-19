// #define _DO_UNIT_TESTING

#include <Arduino.h>

#include "RelayPins.h"

void RelayPins::setup() {
	pinMode(HEAT_RELAY, OUTPUT);
	pinMode(COOL_RELAY, OUTPUT);
}

void RelayPins::setToAction(Action action) {
	//do the action
	switch ( action) {
		
		case REST:
			digitalWrite(HEAT_RELAY, LOW); // turn the Heat off
			digitalWrite(COOL_RELAY, LOW); // turn the Cool off
			break;
			
		case HEAT:
			digitalWrite(HEAT_RELAY, HIGH); // turn the Heat on
			digitalWrite(COOL_RELAY, LOW); // turn the Cool off
			break;
			
		case COOL:
			digitalWrite(HEAT_RELAY, LOW); // turn the Heat off
			digitalWrite(COOL_RELAY, HIGH); // turn the Cool on
			break;
			
		default:
			break;
	}	
}

#ifdef _DO_UNIT_TESTING
#include <AUnit.h>
test(SetRelayAction) {
	
	RelayPins::setup();
	RelayPins::setToAction(HEAT);
	assertEqual(HIGH,digitalRead(HEAT_RELAY));
	assertEqual(LOW,digitalRead(COOL_RELAY));
	
	RelayPins::setToAction(COOL);
	assertEqual(LOW,digitalRead(HEAT_RELAY));
	assertEqual(HIGH,digitalRead(COOL_RELAY));

	RelayPins::setToAction(REST);
	assertEqual(LOW,digitalRead(HEAT_RELAY));
	assertEqual(LOW,digitalRead(COOL_RELAY));
	
}

#endif
