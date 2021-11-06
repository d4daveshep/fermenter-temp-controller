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
#endif
