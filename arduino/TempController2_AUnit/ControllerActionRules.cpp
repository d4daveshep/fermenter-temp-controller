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
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	// Test 2. regardless of what we're currently doing, we are above our target range and have natural heating so start cooling
	if(actual > getTargetRangeMax() && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}

	// Test 4. Regardless of what we're doing, when ambient is high and we are below target range, just REST and use natural heating
	if( actual < getTargetRangeMin() && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
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
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}
	
	// Test 3.2 we're already cooling within our target range and have natural heating so keep cooling
	if( now == COOL && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
		return decision;
	}
	
	// Test 3.3 we're heating within our target range and have natural heating so stop and rest
	if( now == HEAT && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
		decision.setNextAction(NO_ACTION);
		decision.setReasonCode("NO_RC");
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
	assertEqual(decision.getNextAction(), HEAT);
	assertEqual(decision.getReasonCode(), "RC1");
	
	// Test 1.2 we are cooling and ambient is high but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual(decision.getNextAction(), HEAT);
	assertEqual(decision.getReasonCode(), "RC1");
	
	// Test 1.3 we are heating and ambient is high but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual(decision.getNextAction(), HEAT);
	assertEqual(decision.getReasonCode(), "RC1");
	

	/*
	* Test 2. ambient is high, we are resting | cooling | heating but temp is below target range.  REST, REST, REST
	// Test 2.1 we are resting and ambient is high and temp is below target range so REST and use natural heating
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
	assertEqual(nextAction, REST);
	assertEqual("Reason: below target range, have natural heating, so rest and use natural heating", controller.getReason());

	// Test 2.2 we are cooling and ambient is high but temp is below target range so REST and use natural heating
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
	assertEqual(nextAction, REST);
	assertEqual("Reason: below target range, have natural heating, so stop cooling use natural heating", controller.getReason());
	
	// Test 2.3 we are heating and ambient is high but temp is below target range so REST and use natural heating
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
	assertEqual(nextAction, REST);
	assertEqual("Reason: below target range, have natural heating, so rest and use natural heating", controller.getReason());
	*/
	
	/*
	* Test 3. ambient is high, we are resting | cooling | heating but temp is within target range.  REST, COOL, REST
	// Test 3.1 we are resting and ambient is high but temp is within target range so keep RESTing
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
	assertEqual(nextAction, REST);
	assertEqual("Reason: within target range, have natural heating, so keep resting", controller.getReason());
	
	// Test 3.2 we are cooling and ambient is high but temp is within target range so keep COOLing
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: within target range, have natural heating, but will keep cooling", controller.getReason());
	
	// Test 3.3 we are heating (why!) and ambient is high but temp is within target range so REST
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
	assertEqual(nextAction, REST);
	assertEqual("Reason: within target range, have natural heating, so stop heating and rest", controller.getReason());
	*/
	
	/*
	* Test 4. ambient is high, we are resting | cooling | heating but temp is above target range.  COOL, COOL, COOL
	// Test 4.1 we are resting and ambient is high but temp is above target range so start COOLing
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above target range, have natural heating, so start cooling", controller.getReason());
	
	// Test 4.2 we are cooling and ambient is high but temp is above target range so keep COOLing
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above target range, have natural heating, so keep cooling", controller.getReason());
	
	// Test 4.3 we are heating (why!) and ambient is high but temp is above target range so COOL
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above target range, have natural heating, so start cooling", controller.getReason());
	*/
	

	/*
	* Test 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	// Test 5.1 we are resting and ambient is high but temp is above failsafe so COOL
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above failsafe temp, so start cooling", controller.getReason());
	
	// Test 5.2 we are cooling and ambient is high but temp is above failsafe so COOL
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above failsafe temp, so keep cooling", controller.getReason());
	
	// Test 5.3 we are heating and ambient is high but temp is above failsafe so COOL
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
	assertEqual(nextAction, COOL);
	assertEqual("Reason: above failsafe temp, so start cooling", controller.getReason());
	*/
	

	/*
	* Test 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	// Test 6.1 we are resting and ambient is low and temp is below failsafe so HEAT
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowFailsafe);
	assertEqual(nextAction, HEAT);

	// Test 6.2 we are cooling and ambient is low but temp is below failsafe so HEAT
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowFailsafe);
	assertEqual(nextAction, HEAT);

	// Test 6.3 we are heating and ambient is low but temp is below failsafe so HEAT
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowFailsafe);
	assertEqual(nextAction, HEAT);
	*/
	
	/*
	* Test 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
	// Test 7.1 we are resting and ambient is low and temp is below target range so HEAT to counteract natural cooling
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowTargetRange);
	assertEqual(nextAction, HEAT);

	// Test 7.2 we are cooling and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowTargetRange);
	assertEqual(nextAction, HEAT);

	// Test 7.3 we are heating and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientLow, belowTargetRange);
	assertEqual(nextAction, HEAT);
	*/
	
	/*
	* Test 8. ambient is low, we are resting | cooling | heating but temp is within target range. REST, REST, HEAT
	// Test 8.1 we are resting and ambient is low but temp is within target range so keep RESTing
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientLow, withinTargetRange);
	assertEqual(nextAction, REST);

	// Test 8.2 we are cooling and ambient is low but temp is within target range so REST
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientLow, withinTargetRange);
	assertEqual(nextAction, REST);

	// Test 8.3 we are heating and ambient is low but temp is within target range so keep HEATing
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientLow, withinTargetRange);
	assertEqual(nextAction, HEAT);
	*/
	
	/*
	* Test 9. ambient is low, we are resting | cooling | heating but temp is above target range. REST, REST, REST 
	// Test 9.1 we are resting and ambient is low but temp is above target range so keep RESTing and use natural cooling
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveTargetRange);
	assertEqual(nextAction, REST);

	// Test 9.2 we are cooling and ambient is low but temp is above target range so REST (and use natural cooling
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveTargetRange);
	assertEqual(nextAction, REST);

	// Test 9.3 we are heating and ambient is low but temp is above target range so REST (and use natural cooling)
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveTargetRange);
	assertEqual(nextAction, REST);
	*/
	
	/*
	* Test 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	// Test 10.1 we are resting and ambient is low but temp is above failsafe so COOL
	currentAction = REST;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveFailsafe);
	assertEqual(nextAction, COOL);

	// Test 10.2 we are cooling and ambient is low but temp is above failsafe so COOL
	currentAction = COOL;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveFailsafe);
	assertEqual(nextAction, COOL);

	// Test 10.3 we are heating and ambient is low but temp is above failsafe so COOL
	currentAction = HEAT;
	nextAction = controller.getNextAction(currentAction, ambientLow, aboveFailsafe);
	assertEqual(nextAction, COOL);
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


