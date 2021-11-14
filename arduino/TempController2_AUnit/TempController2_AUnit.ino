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
    double latestTemperatureReading = 0.0;
    unsigned long countNumberOfReadings = 0;
    
public:
    TemperatureReadings(int numberOfReadingsUsedForAverage) {
        setNumberOfReadingsUsedForAverage(numberOfReadingsUsedForAverage);
    }
    void clear() {
        this->averageTemperature = 0.0;
        this->latestTemperatureReading = 0.0;
        this->countNumberOfReadings = 0;
    }
    int getNumberOfReadingsUsedForAverage() {
        return this->numberOfReadingsUsedForAverage;
    }
    void setNumberOfReadingsUsedForAverage(int numberOfReadingsUsedForAverage) {
        this->numberOfReadingsUsedForAverage = numberOfReadingsUsedForAverage;
    }
    double getCurrentAverageTemperature() {
        return this->averageTemperature;
    }
    double getLatestTemperatureReading() {
        return this->latestTemperatureReading;
    }
    unsigned long getCountOfTemperatureReadings() {
        return this->countNumberOfReadings;
    }
    
    /*
     * Calculate exponential moving average (EMA)
     * new average = (old average * (n-1) + new value) / n
     */
    void updateAverageTemperatureWithNewValue(double newTemperatureReading) {
        this->latestTemperatureReading = newTemperatureReading;
        double newAverage = ( averageTemperature * (numberOfReadingsUsedForAverage-1) + newTemperatureReading ) / numberOfReadingsUsedForAverage;
        this->averageTemperature = newAverage;
        this->countNumberOfReadings++;
    }
    
    double generateRandomTemperature(int min, int max) {
        double temp = random( min, max ) + random( 0, 100 ) / 100.0;
        Serial.print("random temp = ");
        Serial.print(temp);
        Serial.print("\n");
        return temp;
    }
    
    double randomTempMovement() {
        return random(-10,11) / 100.0;
    }

};


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

testF(TemperatureReadingsTest, MinAndMaxTemperatures) {
    
    randomSeed(1);
    randomSeed(0);
    generateRandomReadings(10);
    assertEqual((unsigned long)10, (unsigned long)(temperatureReadings.getCountOfTemperatureReadings()));
    assertEqual(12.14163,  temperatureReadings.getCurrentAverageTemperature());
    
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
    randomSeed(1);
    randomSeed(0);
    //TemperatureReadings temperatureReadings(10);
    temperatureReadings.clear();
    //int numberOfReadingsUsedForAverage = temperatureReadings.getNumberOfReadingsUsedForAverage();
    
    // first 10 values are 22.49, 18.58, 15.72, 19.78, 18.09, 15.65, 17.42, 22.03, 22.29, 15.12, 
    // true average is 18.717, exponential moving average is 12.14163 
    
    // average starts at 0.0
    //assertEqual(0.0, temperatureReadings.getCurrentAverageTemperature());
    
    for( int i=0; i<10; i++ ) {
        temperatureReadings.updateAverageTemperatureWithNewValue(temperatureReadings.generateRandomTemperature(15,25));
    }
    assertEqual(12.14163,  temperatureReadings.getCurrentAverageTemperature());
}   

test(ExponentialMovingAverageOfTemperatureReadings) {
    randomSeed(0); // setup the random number generator to a predictable sequence
    //TemperatureReadings temperatureReadings(10);
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
    
    

}

void loop() {
#ifdef _DO_UNIT_TESTING
	
	aunit::TestRunner::run();

#endif
}
