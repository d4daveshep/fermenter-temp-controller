#define _DO_UNIT_TESTING

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

#ifdef _DO_UNIT_TESTING
void doSomeSlowThing() {
	delay(123);
}

test(SmartDelay) {
	
	SmartDelay smartDelay(1000);
	
	long unsigned start = millis();
	smartDelay.start();
	doSomeSlowThing();
	long timeTaken = smartDelay.doDelay();
	long unsigned end = millis();
	
	assertNear( (unsigned long)1000, (long unsigned)timeTaken, (unsigned long)1 );
	assertNear(start, end, (long unsigned)(1000+1));
	
}
#endif
