#ifndef _FERMENTATION_PROFILE_H
#define _FERMENTATION_PROFILE_H

#include <Arduino.h>

class FermentationProfile {

  private:
    String name;
    double temp, range;
    bool valid = false;
    
  public:

    FermentationProfile(); 
    FermentationProfile(String name, double temp, double range);

    String getName();

    double getFermentationTemp();
    double setFermentationTemp(double new_temp);
    
    double getTemperatureRange();

    bool isValid();
};

#endif
