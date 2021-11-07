#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "ControllerActionRules.h"
#include "Decision.h"
// #include "ControllerTestOnce.cpp"

ControllerActionRules::ControllerActionRules(double targetTemp, double targetRange) {
	this->target = targetTemp;
	this->range = targetRange;
}

double ControllerActionRules::getFailsafeMin() {
	return this->target - ( this->range * 2.0 );
}

double ControllerActionRules::getFailsafeMax() {
	return this->target + ( this->range * 2.0 );
}

double ControllerActionRules::getTargetRangeMin() {
	return this->target - this->range;
}

double ControllerActionRules::getTargetRangeMax() {
	return this->target + this->range;
}

double ControllerActionRules::getTargetTemp() {
	return this->target;
}

void ControllerActionRules::setTargetTemp(double newTargetTemp) {
	this->target = newTargetTemp;
}

bool ControllerActionRules::inTargetRange(double temp) {
	return getTargetRangeMin() <= temp && temp <= getTargetRangeMax();
}


/*
* Natural Drift hapens is when ambient temp is different to actual fermenter temp.
* When ambient > actual we have natural heating
* When ambient < actual we have natural cooling
* Note:  target temp is irrelevant
*/
NaturalDrift ControllerActionRules::getNaturalDrift(double ambient, double actual) {

	// if ambient is below actual then assume natural cooling will happen
	if(ambient < actual) {
		return NATURAL_COOLING;
	}

	// if ambient is above actualthen assume natural heating will happen
	if(ambient >= actual) {
		return NATURAL_HEATING;
	}

	return DRIFT_ERROR;
}

Decision ControllerActionRules::getActionDecision( Action now, double ambient, double actual ) {
	
	Decision decision;
	
	// Test 1,6. if we've tripped failsafe then disregard ambient and current action
	if( actual < getFailsafeMin() ) {
		decision.setNextAction(HEAT);
		decision.setReasonCode("RC1");
		return decision;
	}

	// Test 5,10. if we've tripped failsafe then disregard ambient and current action
	if( actual > getFailsafeMax() ) {
		decision.setNextAction(COOL);
		decision.setReasonCode("RC5");
		return decision;
	}

	// Rules for when there is NATURAL_HEATING
	if( getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		/*
		 *	| RC2.1 | REST->HEAT because even though there is natural heating, the temperature is below the target range |
		 *	| RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
		 *	| RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
		 */
		if( actual < getTargetRangeMin() ) {
			if( now == REST) {
				decision.setNextAction(HEAT);
				decision.setReasonCode("RC2.1");
			} else if(now == COOL) {
				decision.setNextAction(REST);
				decision.setReasonCode("RC2.2");
			} else if(now == HEAT) {
				decision.setNextAction(REST);
				decision.setReasonCode("RC2.3");
			} else {
				decision.setNextAction(ACTION_ERROR);
				decision.setReasonCode("RC_ERR");
			}
			return decision;
		}
		/*
		 *	| RC4.1 | REST->COOL because the temperature is above the target range and we have natural heating |
		 *	| RC4.2 | COOL->COOL becuase the temperature is above the target range and we have natural heating |
		 *	| RC4.3 | HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?) |
		 */
		if( actual > getTargetRangeMax() ) {
			if( now == REST) {
				decision.setNextAction(COOL);
				decision.setReasonCode("RC4.1");
			} else if(now == COOL) {
				decision.setNextAction(COOL);
				decision.setReasonCode("RC4.2");
			} else if(now == HEAT) {
				decision.setNextAction(COOL);
				decision.setReasonCode("RC4.3");
			} else {
				decision.setNextAction(ACTION_ERROR);
				decision.setReasonCode("RC_ERR");
			}
			return decision;
		}
		
	}

	// Test 7.  Regardless of what we're currently doing, we are below our target range and have natural cooling so start heating
	if(actual < getTargetRangeMin() && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	// Test 9. Regardless of what we're doing, when ambient is low, but temp is above target range, we REST and use natural cooling
	if(actual > getTargetRangeMax() && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	
	
	// Test 3.1 we're already resting within our target range and have natural heating so keep resting
	if( now == REST && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(REST);
		decision.setReasonCode("RC3.1");
		return decision;
	}
	
	// Test 3.2 we're already cooling within our target range and have natural heating so keep cooling
	if( now == COOL && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(COOL);
		decision.setReasonCode("RC3.2");
		return decision;
	}
	
	// Test 3.3 we're heating within our target range and have natural heating so stop and rest
	if( now == HEAT && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(REST);
		decision.setReasonCode("RC3.3");
		return decision;
	}

	// Test 8.1 we are resting within our target range and ambient is low so keep RESTing
	if( now == REST && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	// Test 8.2 we are cooling, temp is within target range and ambient is low so REST (and use natural cooling)
	if( now == COOL && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	// Test 8.3 we are heating. temp is within target range and ambient is low so keep HEATing
	if( now == HEAT && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	decision.setNextAction(ACTION_ERROR);
	decision.setReasonCode("ERROR");
	return decision;
	
}




#ifdef _DO_UNIT_TESTING
/*
* AUnit Tests
*/
/*
Test below failsafe
testF(ControllerTestOnce, BelowFailsafe) {
	assertEqual(name, "David");
}
*/

// Test the decision making logic

test(WhatToDoNext) {
	double target = 18.0;
	double range = 0.5; 
	ControllerActionRules controller(target, range);

	/*
	* Target range is 17.5 to 18.5
	* Failsafe is < 17.0 and > 19.0
	*/

	/*
	* What to we do when....
	* 1. ambient is high, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	* 2. ambient is high, we are resting | cooling | heating but temp is below target range.  REST, REST, REST
	* 3. ambient is high, we are resting | cooling | heating but temp is within target range.  REST, COOL, REST
	* 4. ambient is high, we are resting | cooling | heating but temp is above target range.  COOL, COOL, COOL
	* 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	* 
	* 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	* 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
	* 8. ambient is low, we are resting | cooling | heating but temp is within target range. REST, REST, HEAT
	* 9. ambient is low, we are resting | cooling | heating but temp is above target range. REST, REST, REST 
	* 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	* 
	*/
	double ambientHigh = 22.0;
	double ambientLow = 14.0;
	//double ambientNeutral = target;
	double belowFailsafe = 16.5;
	double belowTargetRange = 17.4;
	double withinTargetRange = target;
	double aboveTargetRange = 18.6;
	double aboveFailsafe = 19.5;
	Action currentAction, nextAction;
	Decision decision;

	/*
	* Test 1. ambient is high, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	*/
	// Test 1.1 we are resting and ambient is high but temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);
	
	// Test 1.2 we are cooling and ambient is high but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);
	
	// Test 1.3 we are heating and ambient is high but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);
	
	/*
	 * Test 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	 */
	// Test 5.1 we are resting and ambient is high but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 5.2 we are cooling and ambient is high but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 5.3 we are heating and ambient is high but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);
	
	/*
	 * Test 2. 
	 * | RC2.1 | REST->HEAT because even though there is natural heating, the temperature is below the target range |
	 * | RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	 * | RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
	 */
	// Test 2.1 we are resting and ambient is high and temp is below target range so HEAT and use natural heating
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.1", decision.getReasonCode());
	assertEqual(decision.getNextAction(), HEAT);

	// Test 2.2 we are cooling and ambient is high but temp is below target range so REST and use natural heating
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.2", decision.getReasonCode());
	assertEqual(decision.getNextAction(), REST);
	
	// Test 2.3 we are heating and ambient is high but temp is below target range so REST and use natural heating
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.3", decision.getReasonCode());
	assertEqual(decision.getNextAction(), REST);
	
	/*
	* Test 3. ambient is high, we are resting | cooling | heating but temp is within target range.  REST, COOL, REST
	* | RC3.1 | REST->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	* | RC3.2 | COOL->COOL because we are still within target range and we have natural heating |
	* | RC3.3 | HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	* 
	*/
	// Test 3.1 we are resting and ambient is high but temp is within target range so keep RESTing
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.1", decision.getReasonCode());
	assertEqual(decision.getNextAction(), REST);
	
	// Test 3.2 we are cooling and ambient is high but temp is within target range so keep COOLing
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.2", decision.getReasonCode());
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 3.3 we are heating (why!) and ambient is high but temp is within target range so REST
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.3", decision.getReasonCode());
	assertEqual(decision.getNextAction(), REST);
	
	/*
	* Test 4. ambient is high, we are resting | cooling | heating but temp is above target range.  COOL, COOL, COOL
	* | RC4.1 | REST->COOL because the temperature is above the target range and we have natural heating |
	* | RC4.2 | COOL->COOL becuase the temperature is above the target range and we have natural heating |
	* | RC4.3 | HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?) |
	*/
	// Test 4.1 we are resting and ambient is high but temp is above target range so start COOLing
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.1", decision.getReasonCode());
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 4.2 we are cooling and ambient is high but temp is above target range so keep COOLing
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.2", decision.getReasonCode());
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 4.3 we are heating (why!) and ambient is high but temp is above target range so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.3", decision.getReasonCode());
	assertEqual(decision.getNextAction(), COOL);

	/*
	* Test 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	// Test 6.1 we are resting and ambient is low and temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getNextAction(), HEAT);

	// Test 6.2 we are cooling and ambient is low but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getNextAction(), HEAT);

	// Test 6.3 we are heating and ambient is low but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getNextAction(), HEAT);
	*/
	
	/*
	* Test 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
	// Test 7.1 we are resting and ambient is low and temp is below target range so HEAT to counteract natural cooling
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getNextAction(), HEAT);

	// Test 7.2 we are cooling and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getNextAction(), HEAT);

	// Test 7.3 we are heating and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getNextAction(), HEAT);
	*/
	
	/*
	* Test 8. ambient is low, we are resting | cooling | heating but temp is within target range. REST, REST, HEAT
	// Test 8.1 we are resting and ambient is low but temp is within target range so keep RESTing
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getNextAction(), REST);

	// Test 8.2 we are cooling and ambient is low but temp is within target range so REST
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getNextAction(), REST);

	// Test 8.3 we are heating and ambient is low but temp is within target range so keep HEATing
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getNextAction(), HEAT);
	*/
	
	/*
	* Test 9. ambient is low, we are resting | cooling | heating but temp is above target range. REST, REST, REST 
	// Test 9.1 we are resting and ambient is low but temp is above target range so keep RESTing and use natural cooling
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getNextAction(), REST);

	// Test 9.2 we are cooling and ambient is low but temp is above target range so REST (and use natural cooling
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getNextAction(), REST);

	// Test 9.3 we are heating and ambient is low but temp is above target range so REST (and use natural cooling)
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getNextAction(), REST);
	*/
	
	/*
	* Test 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	// Test 10.1 we are resting and ambient is low but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getNextAction(), COOL);

	// Test 10.2 we are cooling and ambient is low but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getNextAction(), COOL);

	// Test 10.3 we are heating and ambient is low but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getNextAction(), COOL);
	*/
	



	//assertTrue(false); // finishing these tests



	}

	test(AmbientTempGivesNaturalCoolingOrHeating) {

	double target = 18.0;
	double range = 0.5;
	ControllerActionRules controller(target, range);


	// Test natural cooling
	double ambient = 18.0;
	double actual = 20.0;
	assertEqual( controller.getNaturalDrift( ambient, actual ), NATURAL_COOLING );

	// Test natural heating
	ambient = 18.0;
	actual = 16.0;
	assertEqual( controller.getNaturalDrift( ambient, actual ), NATURAL_HEATING );

	// Test neutral = natural heating
	ambient = 18.0;
	actual = 18.0;
	assertEqual( controller.getNaturalDrift( ambient, actual ), NATURAL_HEATING );




}



#endif


