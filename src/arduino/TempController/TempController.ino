// #line 2 "TempController.ino"

/*
	Fermenter Temp Controller
*/

//#define _DO_UNIT_TESTING

#ifdef _DO_UNIT_TESTING
#warning "Doing Unit Testing Only"
#include <AUnit.h>
#include <ArduinoJson.h>
#include "Decision.h"
#else

#include <LiquidCrystal.h>
#include <OneWire.h>
#include <DallasTemperature.h>
#include <ArduinoJson.h>

#include "Decision.h"
#include "ControllerActionRules.h"
#include "TemperatureReadings.h"
#include "RelayPins.h"
#include "SmartDelay.h"

// initialise the OneWire sensors
const int ONE_WIRE_BUS = 3;  // Data wire is plugged into pin 3 on the Arduino
OneWire oneWire(ONE_WIRE_BUS);  // Setup a oneWire instance to communicate with any OneWire devices
DallasTemperature sensors(&oneWire);  // Pass our oneWire reference to Dallas Temperature.

unsigned long lastJsonPrintTimestamp = 0.0; // timestamp of last serial print

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received

// initialize the LCD library with the numbers of the interface pins
LiquidCrystal lcd(8, 9, 4, 5, 6, 7);  // select the pins used on the LCD panel
char buf[6]; // char buffer used to convert numbers to strings to write to lcd

/*
* NEW GLOBAL VARIABLES
* Create a global instance of our new controller class
*/
TemperatureReadings fermenterTemperatureReadings(60), ambientTemperatureReadings(10);
double defaultTargetTemp = 20.0;
double defaultRange = 0.3; // i.e. +/- either side of target
ControllerActionRules controller(defaultTargetTemp, defaultRange);
Decision decision;
Action currentAction = REST;
StaticJsonDocument<150> jsonDoc;
SmartDelay smartDelay(1000);

#endif

/*
Setup runs once
*/
void setup(void) {
	delay(1000); // wait for stability on some boards to prevent garbage Serial
	Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
	
#ifdef _DO_UNIT_TESTING
	Serial.println();
#else
	// TODO try higher number 115200??
	Serial.begin(115200);

	RelayPins::setup();

	// set up the LCD as 16 x 2
	lcd.begin(16, 2);

	sensors.begin(); // set up the temp sensors
	sensors.requestTemperatures();  // read the temp sensors
	
	// take the first fermenter reading and initialise the average
	double firstFermenterTemperatureReading = sensors.getTempCByIndex(0);
	fermenterTemperatureReadings.setInitialAverageTemperature(firstFermenterTemperatureReading);

	// take the first ambient reading and initialise the average
	double firstAmbientTemp = sensors.getTempCByIndex(1);
	ambientTemperatureReadings.setInitialAverageTemperature(firstAmbientTemp);
	
	lastJsonPrintTimestamp = millis(); // initialise the Json printing timer

	delay(1000); // wait 1 sec before proceeding
#endif
}

/*
* Main loop
*/
void loop(void) {

#ifdef _DO_UNIT_TESTING
	aunit::TestRunner::run();
#else
	
	// start the 1 sec smart delay timer
	smartDelay.start();
	
	// read any data from the serial port
	readSerialWithStartEndMarkers();
	double newTargetTemp = getUpdatedTargetTemp();
	if( newTargetTemp > 0) {
		controller.setTargetTemp(newTargetTemp);
	}

	// do the temp readings and average calculation
	doTempReadings();

	// use our new ControllerActionRules class to determine the next action
	double ambientTemp = ambientTemperatureReadings.getCurrentAverageTemperature();
	double fermenterTemp = fermenterTemperatureReadings.getCurrentAverageTemperature();
	
	decision = controller.makeActionDecision( currentAction, ambientTemp, fermenterTemp );
	Action nextAction = decision.getNextAction();
	currentAction = nextAction;

	// set the relay pins to do the action
	RelayPins::setToAction( currentAction );

	// update the display
	updateLCD();

	// print Json to serial port every 10 secs
	printJsonEvery10Secs();

	// complete the 1 sec smart delay
	smartDelay.doDelay();
	
#endif

}

#ifdef _DO_UNIT_TESTING
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

	String output = "";
	serializeJson(jsonDoc, output);
	assertEqual("{\"now\":12.34,\"avg\":23.45,\"override\":true,\"action\":\"Rest\",\"rest\":true,\"reason-code\":\"RC3.1\"}",output);
}
#else

void printJsonEvery10Secs() {
	unsigned long now = millis();
	if ( ( now - lastJsonPrintTimestamp ) > 9900 ) { // every 10 secs
		printJson();
		lastJsonPrintTimestamp = now;
	}
}

/*
Print Json format to Serial port
*/
void printJson() {
	
	jsonDoc.clear();
	jsonDoc["instant"] = fermenterTemperatureReadings.getLatestTemperatureReading();
	jsonDoc["average"] = fermenterTemperatureReadings.getCurrentAverageTemperature();
	jsonDoc["min"] = fermenterTemperatureReadings.getMinimumTemperature();
	jsonDoc["max"] = fermenterTemperatureReadings.getMaximumTemperature();
	jsonDoc["target"] = controller.getTargetTemp();
	jsonDoc["ambient"] = ambientTemperatureReadings.getCurrentAverageTemperature();
	
	jsonDoc["action"] = decision.getActionText();

	// add booleans for Grafana to use
	switch ( decision.getNextAction() ) {
		case REST:
			jsonDoc["rest"] = true;
			break;
		case HEAT:
			jsonDoc["heat"] = true;
			break;
		case COOL:
			jsonDoc["cool"] = true;
			break;
		default:
			break;
	}

	jsonDoc["reason-code"] = decision.getReasonCode();
//	jsonDoc["timestamp"] = millis();
	jsonDoc["json-size"] = jsonDoc.memoryUsage();

	serializeJson(jsonDoc, Serial);
	Serial.println();
}

/*
Read the serial port using start and end markers: < >
*/
void readSerialWithStartEndMarkers() {
	static boolean recvInProgress = false;
	static byte ndx = 0;
	char startMarker = '<';
	char endMarker = '>';
	char rc;

    newSerialDataReceived = false;

	// if (Serial.available() > 0) {
	while (Serial.available() > 0 && newSerialDataReceived == false) {
		rc = Serial.read();

		if (recvInProgress == true) {
			if (rc != endMarker) {
				receivedChars[ndx] = rc;
				ndx++;
				if (ndx >= serialBufSize) {
				ndx = serialBufSize - 1;
				}
			}
			else {
				receivedChars[ndx] = '\0'; // terminate the string
				recvInProgress = false;
				ndx = 0;
				newSerialDataReceived = true;
			}
		}

		else if (rc == startMarker) {
			recvInProgress = true;
		}
	}
}

/*
* Get the updated target temp or -1 if no updated target temp
*/
double getUpdatedTargetTemp() {
	if (newSerialDataReceived == true) {
		float newTarget = atof(receivedChars);
		if ( newTarget != 0.0 ) {
		return newTarget;
		}
		newSerialDataReceived = false;
	}
	return -1.0;
}


/*
Read the temperature sensors and calculate the averages, update the min and max
*/
void doTempReadings() {

	sensors.requestTemperatures();  // read the sensors
	double fermenterTemp = sensors.getTempCByIndex(0); // read the temp
	double ambientTemp = sensors.getTempCByIndex(1); // read the ambient temp

	// new code to keep
	fermenterTemperatureReadings.updateAverageTemperatureWithNewValue(fermenterTemp);
	ambientTemperatureReadings.updateAverageTemperatureWithNewValue(ambientTemp);
	
}

/*
Update the LCD display (16 characters x 2 lines)
|0123456789012345
|----------------|
|N19.9 T20 A15.5 |
|Rest 18.8-22.2  |
|----------------|
*/
void updateLCD() {

	// print LINE 1
	lcd.setCursor(0, 0);
	// current fermenter temp
	lcd.print("N");
	lcd.print( dtostrf(fermenterTemperatureReadings.getCurrentAverageTemperature(), 4, 1, buf) );
	lcd.print(" ");

	// target temp
	lcd.print("T");
	lcd.print( dtostrf(controller.getTargetTemp(), 2, 0, buf) );
	lcd.print(" ");

	// current ambient temp
	lcd.print("A");
	lcd.print( dtostrf(ambientTemperatureReadings.getCurrentAverageTemperature(), 4, 1, buf) );
	lcd.print(" ");

	// print LINE 2
	lcd.setCursor(0, 1);

	// current action
	Action action = decision.getNextAction();
	lcd.print(decision.getActionText());

	// min & max
	lcd.print(" ");
	lcd.print( dtostrf(fermenterTemperatureReadings.getMinimumTemperature(), 4, 1, buf) );
	lcd.print("-");
	lcd.print( dtostrf(fermenterTemperatureReadings.getMaximumTemperature(), 4, 1, buf) );

}

#endif
