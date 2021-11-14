#line 2 "TempController2_Aunit.ino"

#define _DO_UNIT_TESTING

#ifdef _DO_UNIT_TESTING
#include <AUnit.h>
#endif

#include "ControllerActionRules.h"


//----------------------------------------------------------------------------
// setup() and loop()
//----------------------------------------------------------------------------


class TemperatureReadings {

private:
    int numberOfReadingsUsedForAverage;
    double averageTemperature = 0.0;
    
public:
    TemperatureReadings(int numberOfReadingsUsedForAverage) {
        setNumberOfReadingsUsedForAverage(numberOfReadingsUsedForAverage);
    }
    int getNumberOfReadingsUsedForAverage() {
        return this->numberOfReadingsUsedForAverage;
    }
    void setNumberOfReadingsUsedForAverage(int numberOfReadingsUsedForAverage) {
        this->numberOfReadingsUsedForAverage = numberOfReadingsUsedForAverage;
    }
    double getLatestAverageTemperature() {
        return averageTemperature;
    }
    
    /*
     * new average = (old average * (n-1) + new value) / n
     */
    void updateAverageTemperatureWithNewValue(double newTemperatureReading) {
        double newAverage = ( averageTemperature * (numberOfReadingsUsedForAverage-1) + newTemperatureReading ) / numberOfReadingsUsedForAverage;
        averageTemperature = newAverage;
    }
    
    double randomTemp() {
        return random( 15, 25 ) + random( 0, 100 ) / 100.0;
    }
    
    double randomTempMovement() {
        return random(-10,11) / 100.0;
    }

};

test(RandomNextTemp) {

    TemperatureReadings temperatureReadings(10);
    double currentTemp = 20.0;
    
    double tempMovement;
    
    for(int i=0; i<100; i++ ) {
        currentTemp = currentTemp + temperatureReadings.randomTempMovement();
        Serial.print( currentTemp );
        Serial.print("\n");
    }
        
    
    
    assertTrue(false);
    
}

test(AveragingOfTemperatureReadings) {
    
    TemperatureReadings temperatureReadings(10);
    int numberOfReadingsUsedForAverage = temperatureReadings.getNumberOfReadingsUsedForAverage();
    
    // average starts at 0.0
    assertEqual(0.0, temperatureReadings.getLatestAverageTemperature());
    
    // add first value 22.49
    temperatureReadings.updateAverageTemperatureWithNewValue(22.49);
    assertEqual( 22.49/10, temperatureReadings.getLatestAverageTemperature()); // 2.249
    
    // add next value 18.58
    temperatureReadings.updateAverageTemperatureWithNewValue(18.58);
    assertNear( (2.249*9.0+18.58)/10, temperatureReadings.getLatestAverageTemperature(), 0.005); // 3.8821
    
    // add next value 15.72
    temperatureReadings.updateAverageTemperatureWithNewValue(15.72);
    assertNear( (3.8821*9.0+15.72)/10, temperatureReadings.getLatestAverageTemperature(), 0.005); // 5.065589
    
    
    
    
    
    
    
    
    
//     double sum = 0.0;
//     for( int readingNumber=0; readingNumber <  numberOfReadingsUsedForAverage; readingNumber++ ) {
//     
//         double randomTemperature = temperatureReadings.randomTemp();
//         sum = sum + randomTemperature;
//         Serial.print( randomTemperature );
//         Serial.print(", ");
//     }
//     sum = sum / 10;
//     Serial.print("Average=");
//     Serial.print(sum);
//     Serial.print("\n");
    // 22.49, 18.58, 15.72, 19.78, 18.09, 15.65, 17.42, 22.03, 22.29, 15.12, 
    // average is 18.717
    
//     initialise the temp readings to the first reading just taken
// 	for ( int thisReading = 0; thisReading < NUM_READINGS; thisReading++ ) {
// 		tempReadings[thisReading] = firstReading;
// 	}
// 	averageTemp = firstReading;
// 
// 	tempTotal = averageTemp * NUM_READINGS;

//     assertEqual(18.72, temperatureReadings.getLatestAverageTemperature());    
//     assertTrue(false);
    
}

test(UpdateNumberOfTemperatureReadingsUsedForAverage) {

    TemperatureReadings temperatureReadings(10);
    
    assertEqual(10, temperatureReadings.getNumberOfReadingsUsedForAverage());
    temperatureReadings.setNumberOfReadingsUsedForAverage(20);
    assertEqual(20, temperatureReadings.getNumberOfReadingsUsedForAverage());
}


test(UpdatedTargetTempIsSaved) {
    double defaultTargetTemp = 6.0;
    double defaultRange = 0.3; // i.e. +/- either side of target
    ControllerActionRules controller(defaultTargetTemp, defaultRange);

	double newTargetTemp = 7.0;
	controller.setTargetTemp(newTargetTemp);
	
	assertEqual(controller.getTargetTemp(), newTargetTemp );
	
}

void setup() {
	delay(1000); // wait for stability on some boards to prevent garbage Serial
	Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
	while(!Serial); // for the Arduino Leonardo/Micro only

	Serial.print("\n");
    
    randomSeed(0); // setup the random number generator to a predictable sequence

}

void loop() {
#ifdef _DO_UNIT_TESTING
	
	aunit::TestRunner::run();

#endif
}
