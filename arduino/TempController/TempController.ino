/*
 *  Fermenter Temp Controller
 */

#include <LiquidCrystal.h>
#include <OneWire.h>
#include <DallasTemperature.h>

// define we are in simulation mode or sensor mode (simulation mode generates a random temp rather than reading the sensor)
const boolean SIMULATE = false;
//const boolean SIMULATE = true;

// initialise the OneWire sensors
const int ONE_WIRE_BUS = 3;  // Data wire is plugged into pin 3 on the Arduino
OneWire oneWire(ONE_WIRE_BUS);  // Setup a oneWire instance to communicate with any OneWire devices
DallasTemperature sensors(&oneWire);  // Pass our oneWire reference to Dallas Temperature.

// define and initialise the temp data variables
//const int NUM_READINGS = 60; // we're going to average the temp over 60 readings
const int NUM_READINGS = 10; // we're going to average the temp over 10 readings
double tempReadings[NUM_READINGS]; // array to hold temp readings
int tempIndex = 0; // index of the current reading
double tempTotal = 0; // total of all the temp readings
double averageTemp = 0.0; // average temp reading
double currentTemp = 0.0; // current temp reading
long lastPrintTimestamp = 0.0; // timestamp of last serial print
long lastDelayTimestamp = 0.0; // timestamp of last delay reading
double ambientTemp = 0.0; // ambient temp hopefully won't need averaging as it shouldn't change quickly

// define and initialise the temp control tolerances
float targetTemp = 20.0; // set default target temperature of the fermentation chamber - this could be overwritten by serial data
float coolStartTemp; // temp above target we will start cooling
float coolStopTemp; // temp above target we will stop cooling
float heatStartTemp; // temp below target we will start heating
float heatStopTemp; // temp below target we will stop heating
const float TEMP_DIFF = 0.3; // the tolerance we allow before taking action
float heatLag = 0.0;

float minTemp, cycleMinTemp = 1000.0; // min temperature set to a really high value initally
float maxTemp, cycleMaxTemp = -1000.0; // max temperature set to a really low value initally

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received


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

// define the action states and current action
const int REST = 0;
const int HEAT = 1;
const int COOL = 2;
int currentAction = REST; // the current thing we're doing (e.g. REST, HEAT or COOL)
String changeAction = ""; // used to record when our action changes

/*
 * Setup runs once
 */
void setup(void) {
  Serial.begin(9600);

  // set up the relay pins
  pinMode(HEAT_RELAY, OUTPUT);
  pinMode(COOL_RELAY, OUTPUT);

  // set up the LCD as 16 x 2
  lcd.begin(16, 2);

  double firstReading = 0;
  if (!SIMULATE) {
    sensors.begin(); // set up the temp sensors
    sensors.requestTemperatures();  // read the sensors
    firstReading = sensors.getTempCByIndex(0); // read the temp into index location
    ambientTemp = sensors.getTempCByIndex(1); // read the ambient temp
  } else {
    firstReading = randomTemp(); // get a random temp
    ambientTemp = randomTemp(); // get a random temp
  }

  // initialise the temp readings to the first reading just taken
  for ( int thisReading = 0; thisReading < NUM_READINGS; thisReading++ ) {
    tempReadings[thisReading] = firstReading;
  }
  averageTemp = firstReading;
  tempTotal = averageTemp * NUM_READINGS;

  cycleMinTemp = targetTemp - TEMP_DIFF;

  coolStartTemp = targetTemp + TEMP_DIFF;
  coolStopTemp = targetTemp;
  heatStartTemp = targetTemp - TEMP_DIFF;
  heatStopTemp = targetTemp;
  

  lastPrintTimestamp = millis();
  lastDelayTimestamp = millis();

  delay(1000);
}

/*
 * Main loop 
 */
void loop(void) {

  // read the lcd button state and adjust the temperature accordingly
  if (!SIMULATE) {
    checkLCDButtons();
  }

  // read any data from the serial port
  readSerialWithStartEndMarkers();
  updateTargetTemp();

  // do the temp readings and average calculation
  doTempReadings();

  // what are we doing and do we need to change
  switch ( currentAction ) {
    case REST:
      // are we within tolerance
      if ( (averageTemp < heatStartTemp) || (averageTemp < targetTemp - TEMP_DIFF) ) {
        // we are too cold so start heating
        currentAction = HEAT;
        changeAction = "START HEATING";
        cycleMinTemp = averageTemp;
      }
      else if ( averageTemp > coolStartTemp ) {
        // we are too hot so start cooling
        currentAction = COOL;
        changeAction = "START COOLING";
      }
      else {
        // we are within tolerance so keep resting
        currentAction = REST;
        //changeAction = "";
      }
      break;

    case HEAT:
      // have we reached or exceeded our target yet, but we don't want to overshoot
      if ( averageTemp >= heatStopTemp || averageTemp > targetTemp + TEMP_DIFF ) {
        // yes so stop heating and rest
        currentAction = REST;
        changeAction = "STOP HEATING";
        cycleMaxTemp = averageTemp;

        // we've stopped heating so calculate the point at which we should start heating again
        heatLag = heatStartTemp - cycleMinTemp;
        heatStartTemp = targetTemp - TEMP_DIFF + heatLag;
        
      }
      else {
        //changeAction = "";
      }
      // update the cycleMinTemp
      if( averageTemp < cycleMinTemp ) {
        cycleMinTemp = averageTemp;
        //heatlag = 
      }
      break;

    case COOL:
      // have we reached or exceeded our target yet, but we don't wnat to overshoot
      if ( averageTemp <= coolStopTemp ) {
        // yes so stop cooling and rest
        currentAction = REST;
        changeAction = "STOP COOLING";
      }
      else {
        //changeAction = "";
      }
      break;
  }
  
/*
  // MACHINE LEARING BIT!!
  // update the differences in temp from target and tolerance based on current conditions
  if( changeAction == "START HEATING" ) {
    // we've just started heating again so we're near the bottom of the cycle
    
    // aim for heating to start so that the heat lag time takes us just to the tolerance temp
    // so update the temp at which we start heating 
    heatStartTemp = averageTemp; // reset the temp we start heating
    heatLag = heatStartTemp - cycleMinTemp;

    heatStartTempDiff = targetTemp - heatStartTemp - heatLag;
    cycleMinTemp = averageTemp; // record the cycle min temp so we can use it to adjust the point we start heating in the future
    
  }
  else if( changeAction == "STOP HEATING" ) {
    // we've just stopped heating so update the heat lag and temperature to start heating at again
    heatLag = heatStartTemp - cycleMinTemp;
    //heatStartTemp = 
    
  }
*/

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
//    debug();
  }

  // smart delay of 1000 msec
  do {
    // nothing
  } while ((millis() - lastDelayTimestamp) < 1000);
  lastDelayTimestamp = millis();

}

/*
 * Read the lcd button state and adjust the temperature accordingly
 */
void checkLCDButtons(void) {

  lcd_key = read_LCD_buttons();   // read the buttons

  // depending on which button was pushed, we perform an action
  switch ( lcd_key ) {

    case btnRIGHT:
      break;
    case btnLEFT:
      break;
    case btnUP:
      targetTemp += 1.0;  // increase target temp by 1C
      break;
    case btnDOWN:
      targetTemp -= 1.0;  // decrease target temp by 1C
      break;
    case btnSELECT:
      break;
    case btnNONE:
      break;
  }
}

/*
 * Read the LCD buttons
 */
int read_LCD_buttons() {              // read the buttons
  adc_key_in = analogRead(0);       // read the value from the sensor

  // my buttons when read are centered at these valies: 0, 144, 329, 504, 741
  // we add approx 50 to those values and check to see if we are close
  // We make this the 1st option for speed reasons since it will be the most likely result

  if (adc_key_in > 1000) return btnNONE;

  // For V1.0 comment the other threshold and use the one below:
  if (adc_key_in < 50)   return btnRIGHT;
  if (adc_key_in < 195)  return btnUP;
  if (adc_key_in < 380)  return btnDOWN;
  if (adc_key_in < 555)  return btnLEFT;
  if (adc_key_in < 790)  return btnSELECT;

  return btnNONE;                // when all others fail, return this.
}

/* 
 * Print JSON format to Serial port
*/
void printJSON() {

  Serial.print("{\"now\":");
  Serial.print(currentTemp);
  Serial.print(",\"avg\":");
  Serial.print(averageTemp);
//  Serial.print(",\"min\":");
//  Serial.print(minTemp);
  Serial.print(",\"cyclemin\":");
  Serial.print(cycleMinTemp);
  Serial.print(",\"max\":");
  Serial.print(maxTemp);

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

  if( changeAction != "" ) {
    Serial.print(",\"change\":\"");
    Serial.print(changeAction);
    Serial.print("\"");
    changeAction = ""; // reset the changeAction once we're printed it
  }

  Serial.print(",\"heatstart\":");
  Serial.print(heatStartTemp);

  Serial.print(",\"target\":");
  Serial.print(targetTemp);
  Serial.print(",\"ambient\":");
  Serial.print(ambientTemp);
  Serial.print(",\"timestamp\":");
  Serial.print(millis());

  Serial.print("}");
  Serial.println();
}

void debug() {
  Serial.print("DEBUG: ");
  Serial.print("target=");
  Serial.print(targetTemp);

  Serial.print(", tol=");
  Serial.print(TEMP_DIFF);
  
  Serial.print(", avg=");
  Serial.print(averageTemp);

  Serial.print(", action=");
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

  Serial.print(", cycleMin=");
  Serial.print(cycleMinTemp);

    Serial.print(", heatStartTemp=");
    Serial.print(heatStartTemp);

    Serial.print(", heatLag=");
    Serial.print(heatLag);

  Serial.print(", changeAction=");
  Serial.print(changeAction);
  changeAction = ""; // reset the changeAction once we're printed it
  
  Serial.println();
}

/*
 * Produce a random temperature reading between 15 and 25 degrees
 */
double randomTemp() {
  return random( 15, 25 ) + random( 0, 100 ) / 100.0;
}

/*
 * Read the serial port using start and end markers: < >
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
 * Update the target tempe * |aim20.0 now19.9 |
rature
 */
void updateTargetTemp() {
  if (newSerialDataReceived == true) {
    float newTarget = atof(receivedChars);
    if ( newTarget != 0.0 ) {
      targetTemp = newTarget;
    }
    newSerialDataReceived = false;
  }
}

/*
 * Read the temperature sensors and calculate the averages, update the min and max
 */
void doTempReadings() {
  tempTotal -= tempReadings[tempIndex]; // subtract the last temp reading
  if (!SIMULATE) {
    sensors.requestTemperatures();  // read the sensors
    currentTemp = sensors.getTempCByIndex(0); // read the temp
    ambientTemp = sensors.getTempCByIndex(1); // read the ambient temp
  } else {
    currentTemp = randomTemp(); // get a random temp
    //ambientTemp = randomTemp(); // get a random ambient temp
  }
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
 * Update the LCD display (16 characters x 2 lines)
 * |0123456789012345 
 * |----------------|
 * |N19.9 T20 A15.5 |
 * |Rest 18.8-22.2  |
 * |----------------|
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
  lcd.print( dtostrf(targetTemp, 2, 0, buf) );
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

