/*
 *  Fermentation Controller
 *
 *  v2 Changed over to DS18B20 temp sensor and added 60 sec average temp reading.
 *  v3 Corrected min and max by setting average temp to first temp reading.
 *     Improved changed decision logic so that target temp will always be reached before changing action.
 *  v4 Target temp is adjustable using display up/down buttons
 *  v5 Changed Serial output to JSON format
 *     Only print to serial every minute
 *  v6 Added separate JSON flag for heating, cooling, resting actions
 *  v7 Combined randon/simulation version into main sensor version
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
const int NUM_READINGS = 60; // we're going to average the temp over 60 readings
double tempReadings[NUM_READINGS]; // array to hold temp readings
int tempIndex = 0; // index of the current reading
double tempTotal = 0; // total of all the temp readings
double tempAverage = 0.0; // average temp reading
double currentTemp = 0.0; // current temp reading
long lastPrintTimestamp = 0.0; // timestamp of last serial print
long lastDelayTimestamp = 0.0; // timestamp of last delay reading
double ambientTemp = 0.0; // ambient temp hopefully won't need averaging as it shouldn't change quickly

// define and initialise the temp control tolerances
float targetTemp = 20.0; // set default target temperature of the fermentation chamber - this could be overwritten by serial data
float coolStartTempDiff = 0.3; // temp above target we will start cooling
float coolStopTempDiff = 0.0; // temp above target we will stop cooling (-ve means we overrun target temp)
float heatStartTempDiff = 0.3; // temp below target we will start heating
float heatStopTempDiff = 0.0; // temp below target we will stop heating (-ve means we overrun target temp)
//const float TEMP_DIFF = 0.3; // the tolerance we allow before taking action

float minTemp = 1000.0; // min temperature set to a really high value initally
float maxTemp = -1000.0; // max temperature set to a really low value initally

// define variables for reading temperature from serial
const byte serialBufSize = 12; // size of serial char buffer
char receivedChars[serialBufSize]; // array to hold characters received
boolean newSerialDataReceived = false; // let us know when new serial data received


// initialize the LCD library with the numbers of the interface pins
LiquidCrystal lcd(8, 9, 4, 5, 6, 7);  // select the pins used on the LCD panel
const int LIGHT_PIN = 10; // pin 10 controls the backlight

// define some values used by the panel and buttons
int lcd_key     = 0;
int adc_key_in  = 0;
boolean lightOn = true;  // keep track of whether the LCD light is on or off

const int btnRIGHT =  0;
const int btnUP =     1;
const int btnDOWN =   2;
const int btnLEFT =   3;
const int btnSELECT = 4;
const int btnNONE =   5;

// define the pins used by the relays
const int HEAT_RELAY = 11; //  Heating relay pin
const int COOL_RELAY = 12; // Cooling relay pin

// define the action states and current action
const int REST = 0;
const int HEAT = 1;
const int COOL = 2;
int currentAction = REST; // the current thing we're doing (e.g. REST, HEAT or COOL)

char buf[6]; // char buffer used to convert numbers to strings to write to lcd


void setup(void) {
  Serial.begin(9600);

  // set up the relay pins
  pinMode(HEAT_RELAY,OUTPUT);
  pinMode(COOL_RELAY,OUTPUT);

  // set up the LCD as 16 x 2
  lcd.begin(16,2);

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

//  Serial.print("first reading=");
//  printJSON(firstReading);

  // initialise the temp readings to 0
  for ( int thisReading = 0; thisReading < NUM_READINGS; thisReading++ ) {
    tempReadings[thisReading] = firstReading;
    //tempReadings[thisReading] = 0;
  }
  tempAverage = firstReading;
  tempTotal = tempAverage * NUM_READINGS;

  lastPrintTimestamp=millis();
  lastDelayTimestamp = millis();

  delay(1000);
}

void loop(void) {

  // read the lcd button state and adjust the temperature accordingly
  if(!SIMULATE) {
    lcd_key = read_LCD_buttons();   // read the buttons

    // depending on which button was pushed, we perform an action
    switch( lcd_key ) {

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

  // read any data from the serial port
  readSerialWithStartEndMarkers();
  updateTargetTemp();

  // do the temp readings and average calculation
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


/*
  Serial.print("target=");
  Serial.print(targetTemp);
  Serial.print("+/-");
  Serial.print(TEMP_DIFF);
  Serial.print(" temp=");
  Serial.print(tempReadings[tempIndex]);
*/
  tempTotal += tempReadings[tempIndex]; // add the new temp reading to the total
  tempIndex++; // increment the temp index
  // reset the temp index if at the end of the array
  if ( tempIndex >= NUM_READINGS ) {
    tempIndex = 0;
  }

  tempAverage = tempTotal / NUM_READINGS; // calculate the average
/*
  Serial.print(" avg=" );
  Serial.print(tempAverage);
*/
  // update the max and min values TODO after 60 readings
  if (tempAverage > maxTemp) {
    maxTemp = tempAverage;
  }
  if (tempAverage < minTemp) {
    minTemp = tempAverage;
  }

  // what are we doing and do we need to change
  switch ( currentAction ) {
  case REST:
    // are we within tolerance
    if ( tempAverage < (targetTemp-heatStartTempDiff) ) {
      // we are too cold so start heating
      currentAction = HEAT;
    } 
    else if ( tempAverage > (targetTemp+coolStartTempDiff) ) {
      // we are too hot so start cooling
      currentAction = COOL;
    } 
    else {
      // we are within tolerance so keep resting
      currentAction = REST;
    }
    break;

  case HEAT:
    // have we reached or exceeded our target yet, but we don't want to overshoot
    if ( tempAverage >= ( targetTemp - heatStopTempDiff )) {
      // yes so rest
      currentAction = REST;
    }
    break;

  case COOL:
    // have we reached or exceeded our target yet, but we don't wnat to overshoot
    if ( tempAverage <= ( targetTemp + coolStopTempDiff ) ) {
      // yes so rest
      currentAction = REST;
    }
    break;
  }

  // do the action
  switch ( currentAction) {

  case REST:
    digitalWrite(HEAT_RELAY,LOW); // turn the Heat off
    digitalWrite(COOL_RELAY,LOW); // turn the Cool off
    break;

  case HEAT:  
    digitalWrite(HEAT_RELAY,HIGH); // turn the Heat on
    digitalWrite(COOL_RELAY,LOW); // turn the Cool off
    break;

  case COOL:
    digitalWrite(HEAT_RELAY,LOW); // turn the Heat off
    digitalWrite(COOL_RELAY,HIGH); // turn the Cool on
    break;

    //  default:
  }

  //  fahrenheit = celsius * 1.8 + 32.0;
/*
  Serial.print(" min=");
  Serial.print(minTemp);
  Serial.print(" max=");
  Serial.print(maxTemp);
  Serial.print(" ");
*/
  // print line 1  
  lcd.setCursor(0,0);
  lcd.print("aim");
  lcd.print( dtostrf(targetTemp,4,1,buf) );
  lcd.print(" ");

  lcd.print("now");
  lcd.print( dtostrf(tempAverage,4,1,buf) );
  lcd.print(" ");

  //print line 2
  lcd.setCursor(0,1);

  switch ( currentAction) {

  case REST:
    lcd.print("Rest");
//    Serial.print("Rest");
    break;
  case HEAT:
    lcd.print("Heat");
//    Serial.print("Heat");
    break;
  case COOL:
    lcd.print("Cool");
//    Serial.print("Cool");
    break;
  default:
    lcd.print("ERROR");
//    Serial.print("ERROR");

  }

  //  lcd.print("Heat");
  lcd.print(" ");
  //  lcd.print("Lo ");
  lcd.print( dtostrf(minTemp,4,1,buf) );
  lcd.print("-");
  lcd.print( dtostrf(maxTemp,4,1,buf) );

  // print every minute
  long newPrintTimestamp = millis();
  if( ( millis() - lastPrintTimestamp ) > 59600 ) {
    lastPrintTimestamp = millis();
    printJSON(currentTemp, tempAverage);
  }

  // smart delay of 1000 msec
  do {
    // nothing
  } while ((millis() - lastDelayTimestamp) < 1000);
 lastDelayTimestamp = millis(); 
    

}

int read_LCD_buttons(){               // read the buttons
  adc_key_in = analogRead(0);       // read the value from the sensor 

  // my buttons when read are centered at these valies: 0, 144, 329, 504, 741
  // we add approx 50 to those values and check to see if we are close
  // We make this the 1st option for speed reasons since it will be the most likely result

  if (adc_key_in > 1000) return btnNONE; 

  // For V1.1 us this threshold
  /*
    if (adc_key_in < 50)   return btnRIGHT;  
   if (adc_key_in < 250)  return btnUP; 
   if (adc_key_in < 450)  return btnDOWN; 
   if (adc_key_in < 650)  return btnLEFT; 
   if (adc_key_in < 850)  return btnSELECT;  
   */

  // For V1.0 comment the other threshold and use the one below: 
  if (adc_key_in < 50)   return btnRIGHT;  
  if (adc_key_in < 195)  return btnUP; 
  if (adc_key_in < 380)  return btnDOWN; 
  if (adc_key_in < 555)  return btnLEFT; 
  if (adc_key_in < 790)  return btnSELECT;   

  return btnNONE;                // when all others fail, return this.
}

/* print JSON format to Serial port
 * { "now":21.38, "avg":21.45, "min":19.45, "max":22.89, "action":"COOL", "target":20.00, "timestamp":123456789 }
 */
void printJSON(double currentTemp, double averageTemp) {
  Serial.print("{\"now\":");
  Serial.print(currentTemp);
  Serial.print(",\"avg\":");
  Serial.print(averageTemp);
  Serial.print(",\"min\":");
  Serial.print(minTemp);
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
  
  Serial.print(",\"target\":");
  Serial.print(targetTemp);
  Serial.print(",\"ambient\":");
  Serial.print(ambientTemp);
  Serial.print(",\"timestamp\":");
  Serial.print(millis());
  
  Serial.print("}");
  Serial.println();
}

// produce a random temperature reading between 15 and 25 degrees
double randomTemp() {
  return random( 15,25 ) + random( 0, 100 ) / 100.0;
}

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

void updateTargetTemp() {
  if (newSerialDataReceived == true) {
    float newTarget = atof(receivedChars);
    if( newTarget != 0.0 ) {
      targetTemp = newTarget;
    }
    newSerialDataReceived = false;
  }
}
