#ifndef _DECISION_H
#define _DECISION_H

#include <Arduino.h>

enum Action { NO_ACTION, REST, HEAT, COOL, ACTION_ERROR };
// enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class Decision {
private:
	Action action;
	const char* reasonCode;
	
public:
	Action getNextAction();
	void setNextAction(Action nextAction);
    
    const char* getActionText();
	
	const char* getReasonCode();
	void setReasonCode(const char* reasonCode);
	
	boolean isMade();
	void clear();
};

#endif
