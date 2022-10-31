#include <Arduino.h>
//#include <AUnit.h>

#include "TemperatureReadings.h"

TemperatureReadings::TemperatureReadings(int numberOfReadingsUsedForAverage) {
    setNumberOfReadingsUsedForAverage(numberOfReadingsUsedForAverage);
    clear();
}

void TemperatureReadings::clear() {
    this->averageTemperature = 0.0;
    this->latestTemperatureReading = 0.0;
    this->countNumberOfReadings = 0;
    this->minimumTemperatureReading = 1000.0;
    this->maximumTemperatureReading = -1000.0;
}

int TemperatureReadings::getNumberOfReadingsUsedForAverage() {
    return this->numberOfReadingsUsedForAverage;
}
    
void TemperatureReadings::setNumberOfReadingsUsedForAverage(int numberOfReadingsUsedForAverage) {
    this->numberOfReadingsUsedForAverage = numberOfReadingsUsedForAverage;
}

double TemperatureReadings::getCurrentAverageTemperature() {
    return this->averageTemperature;
}
double TemperatureReadings::getLatestTemperatureReading() {
    return this->latestTemperatureReading;
}
unsigned long TemperatureReadings::getCountOfTemperatureReadings() {
    return this->countNumberOfReadings;
}
double TemperatureReadings::getMinimumTemperature() {
    return this->minimumTemperatureReading;
}
double TemperatureReadings::getMaximumTemperature() {
    return this->maximumTemperatureReading;
}

void TemperatureReadings::setInitialAverageTemperature(double initalAverageTemp){
    if( countNumberOfReadings < 1 ) {
        this->averageTemperature = initalAverageTemp;
    }
}

/*
* Calculate exponential moving average (EMA)
* new average = (old average * (n-1) + new value) / n
*/
void TemperatureReadings::updateAverageTemperatureWithNewValue(double newTemperatureReading) {
    this->latestTemperatureReading = newTemperatureReading;
    double newAverage = ( averageTemperature * (numberOfReadingsUsedForAverage-1) + newTemperatureReading ) / numberOfReadingsUsedForAverage;
    this->averageTemperature = newAverage;
    this->countNumberOfReadings++;
    if( newTemperatureReading < this->minimumTemperatureReading ) {
        this->minimumTemperatureReading = newTemperatureReading;
    }
    if( newTemperatureReading > this->maximumTemperatureReading ) {
        this->maximumTemperatureReading = newTemperatureReading;
    }
}

double TemperatureReadings::generateRandomTemperature(int min, int max) {
    double temp = random( min, max ) + random( 0, 100 ) / 100.0;
    return temp;
}

double TemperatureReadings::generateRandomTemperatureMovement() {
    return random(-10,11) / 100.0;
}

