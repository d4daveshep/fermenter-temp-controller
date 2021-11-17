#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

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


#ifdef _DO_UNIT_TESTING
/*
 * AUnit Tests
 */
test(StoresActionAndReason) {
    Action action = HEAT;
	String code = "RC_TEST";
	Decision decision;
	decision.setNextAction(action);
	decision.setReasonCode(code);
	
	assertEqual(decision.getNextAction(), action);
	assertEqual(decision.getReasonCode(), code);
}

test(ActionText) {
	Decision decision;
    
    decision.setNextAction(NO_ACTION);
    assertEqual("No Action", decision.getActionText());

    decision.setNextAction(REST);
    assertEqual("Rest", decision.getActionText());

    decision.setNextAction(HEAT);
    assertEqual("Heat", decision.getActionText());
    
    decision.setNextAction(COOL);
    assertEqual("Cool", decision.getActionText());

    decision.setNextAction(ACTION_ERROR);
    assertEqual("Error", decision.getActionText());
}

#endif
