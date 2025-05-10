#include <Arduino.h>
#include <AUnit.h>

#include "SmartDelay.h"

SmartDelay::SmartDelay(long unsigned msec) {
	this->delayInMSec = msec;
}

void SmartDelay::start() {
	this->startingTimestamp = millis();
}

long SmartDelay::doDelay() {
	do {
		// nothing
	} while ((millis() - this->startingTimestamp) < this->delayInMSec);
	return (millis() - this->startingTimestamp);
}
