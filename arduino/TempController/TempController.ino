/*
	Fermenter Temp Controller
*/

#include <LiquidCrystal.h>
#include <OneWire.h>
#include <DallasTemperature.h>
#include <ArduinoJson.h>


#include "ControllerActionRules.h"
#include "TemperatureReadings.h"


// initialise the OneWire sensors
const int ONE_WIRE_BUS = 3;  // Data wire is plugged into pin 3 on the Arduino
OneWire oneWire(ONE_WIRE_BUS);  // Setup a oneWire instance to communicate with any OneWire devices
DallasTemperature sensors(&oneWire);  // Pass our oneWire reference to Dallas Temperature.

long lastPrintTimestamp = 0.0; // timestamp of last serial print
long lastDelayTimestamp = 0.0; // timestamp of last delay reading

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received

// initialize the LCD library with the numbers of the interface pins
LiquidCrystal lcd(8, 9, 4, 5, 6, 7);  // select the pins used on the LCD panel
char buf[6]; // char buffer used to convert numbers to strings to write to lcd

// define the pins used by the heating and cooling relays
const int HEAT_RELAY = 11; //  Heating relay pin
const int COOL_RELAY = 12; // Cooling relay pin

/*
* NEW GLOBAL VARIABLES
* Create a global instance of our new controller class
*/
TemperatureReadings fermenterTemperatureReadings(10), ambientTemperatureReadings(10);
double defaultTargetTemp = 20.0;
double defaultRange = 0.3; // i.e. +/- either side of target
ControllerActionRules controller(defaultTargetTemp, defaultRange);
Decision decision;
Action currentAction = REST;
StaticJsonDocument<100> jsonDoc;

/*
Setup runs once
*/
void setup(void) {
	// TODO try higher number 115200??
	Serial.begin(9600);

	// set up the relay pins
	pinMode(HEAT_RELAY, OUTPUT);
	pinMode(COOL_RELAY, OUTPUT);

	// set up the LCD as 16 x 2
	lcd.begin(16, 2);

	sensors.begin(); // set up the temp sensors
	sensors.requestTemperatures();  // read the sensors
	
	double firstFermenterTemperatureReading = sensors.getTempCByIndex(0); // read the temp into index location
	double firstAmbientTemp = sensors.getTempCByIndex(1); // read the ambient temp

	// new code to keep - initialise the readings by setting averages to first readings
	fermenterTemperatureReadings.setInitialAverageTemperature(firstFermenterTemperatureReading);
	ambientTemperatureReadings.setInitialAverageTemperature(firstAmbientTemp);
	
	lastPrintTimestamp = millis();
	lastDelayTimestamp = millis();

	delay(1000);
}

/*
* Main loop
*/
void loop(void) {

	// read the lcd button state and adjust the temperature accordingly

	// read any data from the serial port
	readSerialWithStartEndMarkers();
	double newTargetTemp = getUpdatedTargetTemp();
	if( newTargetTemp > 0) {
		controller.setTargetTemp(newTargetTemp);
	}

	// do the temp readings and average calculation
	doTempReadings();

	// do some debugging

	// use our new ControllerActionRules class to determine the next action
	double ambientTemp = ambientTemperatureReadings.getCurrentAverageTemperature();
	double fermenterTemp = fermenterTemperatureReadings.getCurrentAverageTemperature();
	
	decision = controller.getActionDecision( currentAction, ambientTemp, fermenterTemp );
	Action nextAction = decision.getNextAction();
	
	currentAction = nextAction; // TO-DO probably don't need to do this

	// do the action
	switch ( currentAction) {

		case REST:
		digitalWrite(HEAT_RELAY, LOW); // turn the Heat off
		digitalWrite(COOL_RELAY, LOW); // turn the Cool off
		break;

		case HEAT:
		digitalWrite(HEAT_RELAY, HIGH); // turn the Heat on
		digitalWrite(COOL_RELAY, LOW); // turn the Cool off
		break;

		case COOL:
		digitalWrite(HEAT_RELAY, LOW); // turn the Heat off
		digitalWrite(COOL_RELAY, HIGH); // turn the Cool on
		break;

		//  default:
	}

	// update the display
	updateLCD();

	// print JSON to serial port
	long newPrintTimestamp = millis();
	//  if ( ( millis() - lastPrintTimestamp ) > 59600 ) { // every minute
	if ( ( millis() - lastPrintTimestamp ) > 9900 ) { // every 10 secs
		lastPrintTimestamp = millis();
		printJSON();
// 		debug(nextAction);
	}

	// smart delay of 1000 msec
	do {
		// nothing
	} while ((millis() - lastDelayTimestamp) < 1000);
	lastDelayTimestamp = millis();

}

/*
Print JSON format to Serial port
*/
void printJSON() {
	
	jsonDoc.clear();
	jsonDoc["now"] = fermenterTemperatureReadings.getLatestTemperatureReading();
	jsonDoc["avg"] = fermenterTemperatureReadings.getCurrentAverageTemperature();
	jsonDoc["min"] = fermenterTemperatureReadings.getMinimumTemperature();
	jsonDoc["max"] = fermenterTemperatureReadings.getMaximumTemperature();
	
	jsonDoc["action"] = decision.getActionText();
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

	jsonDoc["target"] = controller.getTargetTemp();
	jsonDoc["ambient"] = ambientTemperatureReadings.getCurrentAverageTemperature();
	
	jsonDoc["timestamp"] = millis();
	
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
	// current temp
	lcd.print("N");
	lcd.print( dtostrf(fermenterTemperatureReadings.getCurrentAverageTemperature(), 4, 1, buf) );
	lcd.print(" ");

	// target temp
	lcd.print("T");
	lcd.print( dtostrf(controller.getTargetTemp(), 2, 0, buf) );
	lcd.print(" ");

	// ambient temp
	lcd.print("A");
	lcd.print( dtostrf(ambientTemperatureReadings.getCurrentAverageTemperature(), 4, 1, buf) );
	lcd.print(" ");

	// print LINE 2
	lcd.setCursor(0, 1);

	// current action
	switch ( currentAction) {

		case REST:
		lcd.print("Rest");
		break;
		case HEAT:
		lcd.print("Heat");
		break;
		case COOL:
		lcd.print("Cool");
		break;
		default:
		lcd.print("ERROR");

	}

	// min & max
	lcd.print(" ");
	lcd.print( dtostrf(fermenterTemperatureReadings.getMinimumTemperature(), 4, 1, buf) );
	lcd.print("-");
	lcd.print( dtostrf(fermenterTemperatureReadings.getMaximumTemperature(), 4, 1, buf) );

}


