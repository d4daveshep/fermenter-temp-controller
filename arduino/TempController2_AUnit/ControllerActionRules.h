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
	double coolingOverrunAdjustment = 0.2; // adjust this by observing real life behaviour
	Decision newDecision;
	
	public:
	ControllerActionRules(double targetTemp, double targetRange);
	double getFailsafeMin();
	double getFailsafeMax();
	double getTargetTemp();
	void   setTargetTemp(double newTargetTemp);
	double getTargetRangeMin();
	double getTargetRangeMax();
	double getStopCoolingTemp();
	bool isTempInTargetRange(double temp);
	bool isTempBelowTargetRange(double temp);
	bool isTempAboveTargetRange(double temp);
	bool isTempBelowFailsafe(double temp);
	bool isTempAboveFailsafe(double temp);
	NaturalDrift getNaturalDrift(double ambient, double actual);
	boolean isNaturalHeating(double ambient, double actual);
	Decision getActionDecision( Action currentAction, double ambient, double actual );
	Decision getDecision();
	void resetDecision();
	void checkFailsafeMinAndDecideAction(double actualTemp); // RC1
	void checkFailsafeMaxAndDecideAction(double actualTemp); // RC5
	void checkForCoolingOverrunAndDecideAction(Action currentAction, double actual, NaturalDrift drift); // RC2.2, RC7.2
	void decideActionWhenBelowTargetRange(Action currentAction, NaturalDrift drift); // RC2.1, RC2.3, RC7.1, RC7.3
	void decideActionWhenInTargetRange(Action currentAction, NaturalDrift drift); // RC3.1, RC3.2, RC3.3
	void decideActionWhenAboveTargetRange(Action currentAction, NaturalDrift drift); // RC4.1, RC4.2, RC4.3
	/*
	 * Get the reason for the last decision according to the rules
	 */
// 	String getReason();

};

#endif
