#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "SmartDelay.h"

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
