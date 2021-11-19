#ifndef _DECISION_H
#define _DECISION_H

#include <Arduino.h>

enum Action { NO_ACTION, REST, HEAT, COOL, ACTION_ERROR };

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
	
};

#endif
