#ifndef _CONTROLLER_ACTION_RULES_H
#define _CONTROLLER_ACTION_RULES_H

#include <Arduino.h>
#include "Decision.h"

// enum Action { NO_ACTION, REST, HEAT, COOL, ACTION_ERROR };
enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class ControllerActionRules {

	private:
	double target = 20.0;
	double range = 0.3;
// 	String lastReason = "";
	
	public:
	ControllerActionRules(double targetTemp, double targetRange);
	double getFailsafeMin();
	double getFailsafeMax();
	double getTargetTemp();
	void   setTargetTemp(double newTargetTemp);
	double getTargetRangeMin();
	double getTargetRangeMax();
	bool inTargetRange(double temp);
	NaturalDrift getNaturalDrift(double ambient, double actual);
	Decision getActionDecision( Action currentAction, double ambient, double actual );
	/*
	 * Get the reason for the last decision according to the rules
	 */
// 	String getReason();

};

#endif
