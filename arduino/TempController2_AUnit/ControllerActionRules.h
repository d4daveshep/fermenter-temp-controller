#ifndef _CONTROLLER_ACTION_RULES_H
#define _CONTROLLER_ACTION_RULES_H

#include <Arduino.h>

#include "FermentationProfile.h"

enum Action { ACTION_ERROR, REST, HEAT, COOL };
enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class ControllerActionRules {

  private:
  FermentationProfile profile;
  //double ambientDriftThreshhold = 1.0; // difference between ambient and target that we assume will be influencial

  public:
  ControllerActionRules(FermentationProfile fp);
  double getFailsafeMin();
  double getFailsafeMax();
  double getTargetRangeMin();
  double getTargetRangeMax();
  bool inTargetRange(double temp);
  NaturalDrift getNaturalDrift(double ambient, double actual);
  Action getNextAction( Action currentAction, double ambient, double actual );

};

#endif
