#ifndef _RELAY_PINS_H
#define _RELAY_PINS_H

#include <Arduino.h>

#include "Decision.h"

// define the pins used by the heating and cooling relays
const int HEAT_RELAY = 11; //  Heating relay pin
const int COOL_RELAY = 12; // Cooling relay pin

class RelayPins {

public:
	static void setup();
	static void setToAction(Action action);
};

#endif
