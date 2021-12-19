//#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "TemperatureReadings.h"

#ifdef _DO_UNIT_TESTING
/*
* AUnit Tests
*/

TemperatureReadings temperatureReadings(10);

class TemperatureReadingsTest: public aunit::TestOnce {

protected:
    
    void setup() override {
        TestOnce::setup();
        
        // set up stuff here
        Serial.print("Running setup...\n");
        temperatureReadings.clear(); // just to be safe, zero the averages
    }
    
    void teardown() override {
        // tear down stuff here
        temperatureReadings.clear();
        TestOnce::teardown();
    }
    
    void generateRandomReadings(int numberToGenerate) {
        for( int i=0; i<numberToGenerate; i++ ) {
            temperatureReadings.updateAverageTemperatureWithNewValue(temperatureReadings.generateRandomTemperature(15,25));
        }
    }
    
};

testF(TemperatureReadingsTest, SetInitialAverageTemperature) {

    // check initial average temp is 0.0
    assertEqual(0.0, temperatureReadings.getCurrentAverageTemperature());
    
    // set initial avg temp
    temperatureReadings.setInitialAverageTemperature(12.34);
    
    // check it got saved
    assertEqual(12.34, temperatureReadings.getCurrentAverageTemperature());
    
    // update the average temp
    temperatureReadings.updateAverageTemperatureWithNewValue(13.46);
    
    // try to set inital avg temp again - should fail silently
    temperatureReadings.setInitialAverageTemperature(23.45);

    // check avg temp not changed
    assertEqual((12.34*9+13.46)/10, temperatureReadings.getCurrentAverageTemperature());
    
}

testF(TemperatureReadingsTest, MinAndMaxTemperatures) {
    
    assertEqual(1000.0, temperatureReadings.getMinimumTemperature());
    assertEqual(-1000.0, temperatureReadings.getMaximumTemperature());

    randomSeed(1); randomSeed(0); 
    generateRandomReadings(10);
    
    assertEqual(15.12, temperatureReadings.getMinimumTemperature());
    assertEqual(22.49, temperatureReadings.getMaximumTemperature());
    
}

/*
test(RandomNextTemp) {
    randomSeed(0); // setup the random number generator to a predictable sequence
    TemperatureReadings temperatureReadings(10);
    double currentTemp = 20.0;

    for(int i=0; i<10; i++ ) {
        currentTemp = currentTemp + temperatureReadings.randomTempMovement();
        Serial.print( currentTemp );
        Serial.print("\n");
    }
    
    assertTrue(false);
    
}
*/

testF(TemperatureReadingsTest, EMA) {
   
    randomSeed(1); randomSeed(0); 
    generateRandomReadings(10);
    assertEqual((unsigned long)10, (unsigned long)(temperatureReadings.getCountOfTemperatureReadings()));
    assertEqual(12.14163,  temperatureReadings.getCurrentAverageTemperature());
    
}   

test(ExponentialMovingAverageOfTemperatureReadings) {
    randomSeed(1); randomSeed(0);  // setup the random number generator to a predictable sequence
    
    temperatureReadings.clear();
    //int numberOfReadingsUsedForAverage = temperatureReadings.getNumberOfReadingsUsedForAverage();
    
    // first 10 values are 22.49, 18.58, 15.72, 19.78, 18.09, 15.65, 17.42, 22.03, 22.29, 15.12, 
    // true average is 18.717, exponential moving average is 12.14163 
    
    // average starts at 0.0
    assertEqual(0.0, temperatureReadings.getCurrentAverageTemperature());
    
    for( int i=0; i<10; i++ ) {
        temperatureReadings.updateAverageTemperatureWithNewValue(temperatureReadings.generateRandomTemperature(15,25));
    }
    assertEqual(12.14163,  temperatureReadings.getCurrentAverageTemperature());
    
    /*
    // add first value 22.49
    temperatureReadings.updateAverageTemperatureWithNewValue(22.49);
    assertEqual( 22.49/10, temperatureReadings.getLatestAverageTemperature()); // 2.249
    
    // add next value 18.58
    temperatureReadings.updateAverageTemperatureWithNewValue(18.58);
    assertNear( (2.249*9.0+18.58)/10, temperatureReadings.getLatestAverageTemperature(), 0.005); // 3.8821
    
    // add next value 15.72
    temperatureReadings.updateAverageTemperatureWithNewValue(15.72);
    assertNear( (3.8821*9.0+15.72)/10, temperatureReadings.getLatestAverageTemperature(), 0.005); // 5.065589
    */
      
}

test(UpdateNumberOfTemperatureReadingsUsedForAverage) {

    TemperatureReadings temperatureReadings(10);
    
    assertEqual(10, temperatureReadings.getNumberOfReadingsUsedForAverage());
    temperatureReadings.setNumberOfReadingsUsedForAverage(20);
    assertEqual(20, temperatureReadings.getNumberOfReadingsUsedForAverage());
}




#endif
