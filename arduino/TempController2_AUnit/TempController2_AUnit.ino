#line 2 "TempController2_Aunit.ino"

#define _DO_UNIT_TESTING

#ifdef _DO_UNIT_TESTING
#include <AUnit.h>
#endif

#include <ArduinoJson.h>

#include "ControllerActionRules.h"
#include "TemperatureReadings.h"

StaticJsonDocument<100> jsonDoc;

//String getActionText(Action action) {
//     switch( action ) {
//         case NO_ACTION:
//             return "No Action";
//         case REST:
//             return "Rest";
// 		case HEAT:
//             return "Heat";
// 		case COOL:
//             return "Cool";
//         case ACTION_ERROR:
//             return "Error";
// 		default:
//             return "Unknown Action";
// 	}
// }
// 
// test(ActionText) {
//     Action action = NO_ACTION;
//     assertEqual("No Action", getActionText(action));
//     action = REST;
//     assertEqual("Rest", getActionText(action));
//     action = HEAT;
//     assertEqual("Heat", getActionText(action));
//     action = COOL;
//     assertEqual("Cool", getActionText(action));
//     action = ACTION_ERROR;
//     assertEqual("Error", getActionText(action));
// }

test(WriteJsonString) {
    jsonDoc["now"] = 12.34;
    jsonDoc["avg"] = 23.45;
    jsonDoc["override"] = true;
    Decision decision;
    decision.setNextAction(REST);
    jsonDoc["action"] = decision.getActionText();
    jsonDoc["rest"] = true;
    
    Serial.println(jsonDoc.memoryUsage());
    serializeJson(jsonDoc, Serial);
//     Serial.println("\n");
    
    String output = "";
    serializeJson(jsonDoc, output);
    assertEqual("{\"now\":12.34,\"avg\":23.45,\"override\":true,\"action\":\"Rest\",\"rest\":true}",output);
}

test(UpdatedTargetTempIsSaved) {
    double defaultTargetTemp = 6.0;
    double defaultRange = 0.3; // i.e. +/- either side of target
    ControllerActionRules controller(defaultTargetTemp, defaultRange);

	double newTargetTemp = 7.0;
	controller.setTargetTemp(newTargetTemp);
	
	assertEqual(controller.getTargetTemp(), newTargetTemp );
	
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
