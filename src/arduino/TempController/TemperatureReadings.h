#ifndef _TEMPERATURE_READINGS_H
#define _TEMPERATURE_READINGS_H

#include <Arduino.h>

class TemperatureReadings {

private:
    int numberOfReadingsUsedForAverage;
    double averageTemperature;
    double latestTemperatureReading;
    unsigned long countNumberOfReadings;
    double minimumTemperatureReading;
    double maximumTemperatureReading;
    
public:
    TemperatureReadings(int numberOfReadingsUsedForAverage);
    void clear();
    int getNumberOfReadingsUsedForAverage();
    void setNumberOfReadingsUsedForAverage(int numberOfReadingsUsedForAverage);
    double getCurrentAverageTemperature();
    void setInitialAverageTemperature(double initalAverageTemp);
    double getLatestTemperatureReading();
    unsigned long getCountOfTemperatureReadings();
    double getMinimumTemperature();
    double getMaximumTemperature();
    
    /*
     * Calculate exponential moving average (EMA)
     * new average = (old average * (n-1) + new value) / n
     */
    void updateAverageTemperatureWithNewValue(double newTemperatureReading);
    
    double generateRandomTemperature(int min, int max);
    double generateRandomTemperatureMovement();
    
};

#endif
