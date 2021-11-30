#ifndef _DECISION_H
#define _DECISION_H

#include <Arduino.h>

enum Action { NO_ACTION, REST, HEAT, COOL, ACTION_ERROR };
// enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class Decision {
private:
	Action action;
	String reasonCode;
	
public:
	Action getNextAction();
	void setNextAction(Action nextAction);
    
    String getActionText();
	
	String getReasonCode();
	void setReasonCode(String reasonCode);
	
	boolean isMade();
};

// class ControllerActionRules {
// 
// 	private:
// 	double target = 20.0;
// 	double range = 0.3;
// 	String lastReason = "";
// 	
// 	public:
// 	ControllerActionRules(double targetTemp, double targetRange);
// 	double getFailsafeMin();
// 	double getFailsafeMax();
// 	double getTargetTemp();
// 	void   setTargetTemp(double newTargetTemp);
// 	double getTargetRangeMin();
// 	double getTargetRangeMax();
// 	bool inTargetRange(double temp);
// 	NaturalDrift getNaturalDrift(double ambient, double actual);
// 	Action getNextAction( Action currentAction, double ambient, double actual );
// 	/*
// 	 * Get the reason for the last decision according to the rules
// 	 */
// 	String getReason();
// 
// };

#endif
