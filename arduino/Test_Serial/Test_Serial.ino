// #line 2 "TempController.ino"

/*
	Fermenter Temp Controller
*/


#include <ArduinoJson.h>
#include "SmartDelay.h"


unsigned long lastJsonPrintTimestamp = 0.0; // timestamp of last serial print

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received


/*
* NEW GLOBAL VARIABLES
* Create a global instance of our new controller class
*/
double defaultTargetTemp = 20.0;
StaticJsonDocument<100> jsonDoc;
SmartDelay smartDelay(1000);

double ambientTemp = 15.6;
double fermenterTemp = 21.3;
double targetTemp = defaultTargetTemp;

/*
Setup runs once
*/
void setup(void) {
	delay(1000); // wait for stability on some boards to prevent garbage Serial
	Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
	
	Serial.begin(115200);

	lastJsonPrintTimestamp = millis(); // initialise the Json printing timer

	delay(1000); // wait 1 sec before proceeding
}

/*
* Main loop
*/
void loop(void) {

	// start the 1 sec smart delay timer
	smartDelay.start();
	
	// read any data from the serial port
	readSerialWithStartEndMarkers();
	double newTargetTemp = getUpdatedTargetTemp();
	if( newTargetTemp > 0) {
		targetTemp = newTargetTemp;
	}

	// print Json to serial port every 10 secs
	printJsonEvery10Secs();

	// complete the 1 sec smart delay
	smartDelay.doDelay();

}

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
	jsonDoc["target"] = targetTemp;
	jsonDoc["ambient"] = ambientTemp;
	jsonDoc["fermenter"] = fermenterTemp;
	
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
