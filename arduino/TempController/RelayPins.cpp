#include <Arduino.h>
//#include <AUnit.h>

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

