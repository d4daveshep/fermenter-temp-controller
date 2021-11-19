#line 2 "TempController2_Aunit.ino"

#define _DO_UNIT_TESTING

#ifdef _DO_UNIT_TESTING
#include <AUnit.h>
#endif

#include <ArduinoJson.h>

#include "ControllerActionRules.h"
#include "TemperatureReadings.h"
#include "RelayPins.h"


test(WriteJsonString) {
	StaticJsonDocument<100> jsonDoc;

	jsonDoc["now"] = 12.34;
	jsonDoc["avg"] = 23.45;
	jsonDoc["override"] = true;
	Decision decision;
	decision.setNextAction(REST);
	decision.setReasonCode("RC3.1");
	jsonDoc["action"] = decision.getActionText();
	jsonDoc["rest"] = true;
	jsonDoc["reason-code"] = decision.getReasonCode();
	
	Serial.println(jsonDoc.memoryUsage());
// 	serializeJson(jsonDoc, Serial);
// 	Serial.println();
	
	String output = "";
	serializeJson(jsonDoc, output);
	assertEqual("{\"now\":12.34,\"avg\":23.45,\"override\":true,\"action\":\"Rest\",\"rest\":true,\"reason-code\":\"RC3.1\"}",output);
}

test(UpdatedTargetTempIsSaved) {
	double defaultTargetTemp = 6.0;
	double defaultRange = 0.3; // i.e. +/- either side of target
	ControllerActionRules controller(defaultTargetTemp, defaultRange);

	double newTargetTemp = 7.0;
	controller.setTargetTemp(newTargetTemp);
	
	assertEqual(controller.getTargetTemp(), newTargetTemp );
}


class SmartDelay {
private:
	long unsigned delayInMSec;
	long unsigned startingTimestamp;

public:
	SmartDelay(long unsigned msec) {
		this->delayInMSec = msec;
	}

	void start() {
		this->startingTimestamp = millis();
	}
	
	long doDelay() {
		do {
			// nothing
		} while ((millis() - this->startingTimestamp) < this->delayInMSec);
		return (millis() - this->startingTimestamp);
	}
};

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


//----------------------------------------------------------------------------
// setup() and loop()
//----------------------------------------------------------------------------

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
