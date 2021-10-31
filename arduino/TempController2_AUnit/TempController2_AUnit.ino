#line 2 "TempController2_Aunit.ino"

#define _DO_UNIT_TESTING

#ifdef _DO_UNIT_TESTING
#include <AUnit.h>
#endif

#include "ControllerActionRules.h"


//----------------------------------------------------------------------------
// setup() and loop()
//----------------------------------------------------------------------------

/*
* Test real scenario
*/

double defaultTargetTemp = 6.0;
double defaultRange = 0.3; // i.e. +/- either side of target
ControllerActionRules controller(defaultTargetTemp, defaultRange);


test(UpdatedTargetTempIsSaved) {
	double newTargetTemp = 7.0;
	controller.setTargetTemp(newTargetTemp);
	
	assertEqual(controller.getTargetTemp(), newTargetTemp );
	
}

void setup() {
	delay(1000); // wait for stability on some boards to prevent garbage Serial
	Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
	while(!Serial); // for the Arduino Leonardo/Micro only

	Serial.print("\n");

}

void loop() {
#ifdef _DO_UNIT_TESTING
	
	
	
	aunit::TestRunner::run();

#endif
}
