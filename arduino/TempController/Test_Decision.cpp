#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "Decision.h"

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

test(DecisionIsMade) {
	Decision decision;
	decision.setNextAction(NO_ACTION);
	assertFalse(decision.isMade());
	
	decision.setNextAction(REST);
	assertTrue(decision.isMade());
}

test(Clear) {
	Decision decision;
	decision.setNextAction(HEAT);
	decision.clear();
	assertEqual(NO_ACTION, decision.getNextAction());
}

#endif
