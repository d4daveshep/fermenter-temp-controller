#ifndef _CONTROLLER_ACTION_RULES_H
#define _CONTROLLER_ACTION_RULES_H

#include <Arduino.h>

enum Action { ACTION_ERROR, REST, HEAT, COOL };
enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class ControllerActionRules {

	private:
	double target = 20.0;
	double range = 0.3;
	//double ambientDriftThreshhold = 1.0; // difference between ambient and target that we assume will be influencial

	public:
	ControllerActionRules(double targetTemp, double targetRange);
	double getFailsafeMin();
	double getFailsafeMax();
	double getTargetTemp();
	void   setTargetTemp(double newTargetTemp);
	double getTargetRange();
	double getTargetRangeMin();
	double getTargetRangeMax();
	bool inTargetRange(double temp);
	NaturalDrift getNaturalDrift(double ambient, double actual);
	Action getNextAction( Action currentAction, double ambient, double actual );

};

#endif
