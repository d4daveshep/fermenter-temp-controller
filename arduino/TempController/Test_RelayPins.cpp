#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "RelayPins.h"

#ifdef _DO_UNIT_TESTING
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
