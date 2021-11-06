#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "Decision.h"

Action Decision::getNextAction() {
	return NO_ACTION;
}
void Decision::setNextAction(Action nextAction) {
	
}

String Decision::getReasonCode(){
	return "ERROR";
}

void Decision::setReasonCode(String reasonCode) {
	
}


#ifdef _DO_UNIT_TESTING
/*
 * AUnit Tests
 */

#endif
