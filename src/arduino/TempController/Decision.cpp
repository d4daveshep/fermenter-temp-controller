#include <Arduino.h>
//#include <AUnit.h>

#include "Decision.h"

Action Decision::getNextAction() {
	return this->action;
}
void Decision::setNextAction(Action nextAction) {
	this->action = nextAction;
}

String Decision::getActionText() {
    switch( action ) {
        case NO_ACTION:
            return "No Action";
        case REST:
            return "Rest";
		case HEAT:
            return "Heat";
		case COOL:
            return "Cool";
        case ACTION_ERROR:
            return "Error";
		default:
            return "Unknown Action";
	}
}

String Decision::getReasonCode(){
	return this->reasonCode;
}

void Decision::setReasonCode(String reasonCode) {
	this->reasonCode = reasonCode;
}

boolean Decision::isMade() {
	return this->action != NO_ACTION;
}

void Decision::clear() {
	this->action = NO_ACTION;
}

