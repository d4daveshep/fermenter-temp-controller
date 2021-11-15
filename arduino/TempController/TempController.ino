/*
	Fermenter Temp Controller
*/

#include <LiquidCrystal.h>
#include <OneWire.h>
#include <DallasTemperature.h>

#include "ControllerActionRules.h"
#include "TemperatureReadings.h"


// initialise the OneWire sensors
const int ONE_WIRE_BUS = 3;  // Data wire is plugged into pin 3 on the Arduino
OneWire oneWire(ONE_WIRE_BUS);  // Setup a oneWire instance to communicate with any OneWire devices
DallasTemperature sensors(&oneWire);  // Pass our oneWire reference to Dallas Temperature.

// define and initialise the temp data variables
const int NUM_READINGS = 10; // we're going to average the temp over 10 readings
double tempReadings[NUM_READINGS]; // array to hold temp readings
int tempIndex = 0; // index of the current reading
double tempTotal = 0; // total of all the temp readings
double averageTemp = 0.0; // average temp reading
double currentTemp = 0.0; // current temp reading
double ambientTemp = 0.0; // ambient temp hopefully won't need averaging as it shouldn't change quickly

long lastPrintTimestamp = 0.0; // timestamp of last serial print
long lastDelayTimestamp = 0.0; // timestamp of last delay reading

float minTemp, cycleMinTemp = 1000.0; // min temperature set to a really high value initially
float maxTemp, cycleMaxTemp = -1000.0; // max temperature set to a really low value initially

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received

boolean override = false;

// initialize the LCD library with the numbers of the interface pins
LiquidCrystal lcd(8, 9, 4, 5, 6, 7);  // select the pins used on the LCD panel
const int LIGHT_PIN = 10; // pin 10 controls the backlight

// define some values used by the LCD panel and buttons
int lcd_key     = 0;
int adc_key_in  = 0;
boolean lightOn = true;  // keep track of whether the LCD light is on or off
const int btnRIGHT =  0;
const int btnUP =     1;
const int btnDOWN =   2;
const int btnLEFT =   3;
const int btnSELECT = 4;
const int btnNONE =   5;
char buf[6]; // char buffer used to convert numbers to strings to write to lcd


// define the pins used by the heating and coolingrelays
const int HEAT_RELAY = 11; //  Heating relay pin
const int COOL_RELAY = 12; // Cooling relay pin

Action currentAction = REST;

/*
* NEW GLOBAL VARIABLES
* Create a global instance of our new controller class
*/
TemperatureReadings fermenterTemperatureReadings(10), ambientTemperatureReadings(10);
double defaultTargetTemp = 20.0;
double defaultRange = 0.3; // i.e. +/- either side of target
ControllerActionRules controller(defaultTargetTemp, defaultRange);
Decision decision;

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
    
    // TODO refactor these 2 lines
	double firstFermenterTemperatureReading = sensors.getTempCByIndex(0); // read the temp into index location
	ambientTemp = sensors.getTempCByIndex(1); // read the ambient temp

    // new code to keep - initialise the readings by setting averages to first readings
    fermenterTemperatureReadings.setInitialAverageTemperature(firstFermenterTemperatureReading);
    ambientTemperatureReadings.setInitialAverageTemperature(ambientTemp);
    
    // TODO old code to bin
	// initialise the temp readings to the first reading just taken
	for ( int thisReading = 0; thisReading < NUM_READINGS; thisReading++ ) {
		tempReadings[thisReading] = firstFermenterTemperatureReading;
	}
	averageTemp = firstFermenterTemperatureReading;
	tempTotal = averageTemp * NUM_READINGS;

    
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
	decision = controller.getActionDecision( currentAction, ambientTemp, averageTemp );
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

	Serial.print("{\"now\":");
	Serial.print(currentTemp);
	Serial.print(",\"avg\":");
//	Serial.print(averageTemp); TODO remove this
	Serial.print(fermenterTemperatureReadings.getCurrentAverageTemperature());

//     new TemperatureReadings output
// 	Serial.print(",\"new-avg\":");
// 	Serial.print(fermenterTemperatureReadings.getCurrentAverageTemperature());

	if (override) {
		Serial.print(",\"override\":");
		Serial.print("true");
	}

	switch ( currentAction) {
		case REST:
		Serial.print(",\"action\":\"Rest\"");
		Serial.print(",\"rest\":true");
		break;
		case HEAT:
		Serial.print(",\"action\":\"Heat\"");
		Serial.print(",\"heat\":true");
		break;
		case COOL:
		Serial.print(",\"action\":\"Cool\"");
		Serial.print(",\"cool\":true");
		break;
		default:
		Serial.print(",\"action\":\"ERROR\"");
	}

	Serial.print(",\"reason-code\":\"");
	Serial.print(decision.getReasonCode());
	Serial.print("\"");

	Serial.print(",\"target\":");
	Serial.print(controller.getTargetTemp());
	Serial.print(",\"ambient\":");
//	Serial.print(ambientTemp); TODO removde this
	Serial.print(ambientTemperatureReadings.getCurrentAverageTemperature());

//     new TemperatureReadings outoput
// 	Serial.print(",\"new-ambient\":");
// 	Serial.print(ambientTemperatureReadings.getCurrentAverageTemperature());

	Serial.print(",\"timestamp\":");
	Serial.print(millis());

	Serial.print("}");
	Serial.println();
}

void debug(Action nextAction) {
	Serial.print("DEBUG: ");
	Serial.print("target=");
	Serial.print(controller.getTargetTemp());

	Serial.print(", actual=");
	Serial.print(averageTemp);

	Serial.print(", currentAction=");
	switch ( currentAction) {
	 case REST:
	 Serial.print("REST");
	  break;
	 case HEAT:
	 Serial.print("HEAT");
	  break;
	 case COOL:
	 Serial.print("COOL");
	  break;
	 default:
	 Serial.print("ERROR");
   }

	Serial.print(", nextAction=");
	switch ( nextAction) {
	 case REST:
	 Serial.print("REST");
	  break;
	 case HEAT:
	 Serial.print("HEAT");
	  break;
	 case COOL:
	 Serial.print("COOL");
	  break;
	 default:
	 Serial.print("ERROR");
   }

	Serial.print(", ambient=");
	Serial.print(ambientTemp);

	Serial.println();
}

/*
 * Produce a random temperature reading between 15 and 25 degrees
 */
double randomTemp() {
	return random( 15, 25 ) + random( 0, 100 ) / 100.0;
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
	currentTemp = sensors.getTempCByIndex(0); // read the temp
	ambientTemp = sensors.getTempCByIndex(1); // read the ambient temp

    // new code to keep
    fermenterTemperatureReadings.updateAverageTemperatureWithNewValue(currentTemp);
    ambientTemperatureReadings.updateAverageTemperatureWithNewValue(ambientTemp);
    
    // TODO remove this old code to bottom of function
	tempTotal -= tempReadings[tempIndex]; // subtract the last temp reading   
    tempReadings[tempIndex] = currentTemp; // store it in the index location

	tempTotal += tempReadings[tempIndex]; // add the new temp reading to the total
	tempIndex++; // increment the temp index
	// reset the temp index if at the end of the array
	if ( tempIndex >= NUM_READINGS ) {
		tempIndex = 0;
	}

	averageTemp = tempTotal / NUM_READINGS; // calculate the average

	// update the max and min values
	if (averageTemp > maxTemp) {
		maxTemp = averageTemp;
	}
	if (averageTemp < minTemp) {
		minTemp = averageTemp;
	}

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
	lcd.print( dtostrf(averageTemp, 4, 1, buf) );
	lcd.print(" ");

	// target temp
	lcd.print("T");
	lcd.print( dtostrf(controller.getTargetTemp(), 2, 0, buf) );
	lcd.print(" ");

	// ambient temp
	lcd.print("A");
	lcd.print( dtostrf(ambientTemp, 4, 1, buf) );
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
	lcd.print( dtostrf(minTemp, 4, 1, buf) );
	lcd.print("-");
	lcd.print( dtostrf(maxTemp, 4, 1, buf) );

}


