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

double ControllerActionRules::getStopCoolingTemp() {
	return getTargetRangeMin() + this->coolingOverrunAdjustment;
}

bool ControllerActionRules::isTempInTargetRange(double temp) {
	return getTargetRangeMin() <= temp && temp <= getTargetRangeMax();
}

bool ControllerActionRules::isTempBelowTargetRange(double temp) {
	return temp < getTargetRangeMin();
}

bool ControllerActionRules::isTempAboveTargetRange(double temp) {
	return temp > getTargetRangeMax();
}

bool ControllerActionRules::isTempBelowFailsafe(double temp) {
	return temp < getFailsafeMin();
}

bool ControllerActionRules::isTempAboveFailsafe(double temp) {
	return temp > getFailsafeMax();
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

	// Test 1 & 6. if we've tripped failsafe then disregard ambient and current action
	decision.clear();
	TODO think i need to pass the decision object into the check function
	decision = checkFailsafeMinAndDecideAction(actual);
	if( decision.isMade() ) {
		return decision;
	}
	
	// Test 5 & 10. if we've tripped failsafe then disregard ambient and current action
	decision.clear();
	decision = checkFailsafeMaxAndDecideAction(actual);
	if( decision.isMade() ) {
		return decision;
	}

	/*
	 *	| RC2.1 | REST->REST because even though there is natural heating, the temperature is below the target range |
	 *	| RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	 *	| RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
	 */
	if( isNaturalHeating(ambient, actual)) {
		
		decision = checkForCoolingOverrunWithNaturalHeatingAndDecideAction(now, actual); // RC2.2
		if( decision.isMade() ) {
			return decision;
		}
		
		Serial.println("got here 1");
		
		if( actual < getTargetRangeMin() ) {
			Serial.println("got here 2");		
			if( now == REST) {
				Serial.println("got here 3");		
				decision.setNextAction(REST);
				decision.setReasonCode("RC2.1");
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
		 *	| RC3.1 | REST->REST because we are in the target range.  There is natural heating so expect temperature to rise |
		 *	| RC3.2 | COOL->COOL because we are still within target range and we have natural heating |
		 *	| RC3.3 | HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise |
		 */
		if( isTempInTargetRange(actual) ) {
			if( now == REST) {
				decision.setNextAction(REST);
				decision.setReasonCode("RC3.1");
			} else if(now == COOL) {
				decision.setNextAction(COOL);
				decision.setReasonCode("RC3.2");
			} else if(now == HEAT) {
				decision.setNextAction(REST);
				decision.setReasonCode("RC3.3");
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

	if( decision.isMade() ) {
		return decision;
	}

	

	// Rules for when there is NATURAL_COOLING
	if( getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
		/*
		 * | RC7.1 | REST->HEAT because the temperature is below target range and there is natural cooling |
		 * | RC7.2 | COOL->HEAT because the temperature is below the target range and there is natural cooli*ng (adjust cooling lag?) |
		 * | RC7.3 | HEAT->HEAT because the temperature is below target range and there is natural cooling |
		 */
		if(now == COOL && actual < getStopCoolingTemp() ) {
			decision.setNextAction(HEAT);
			decision.setReasonCode("RC7.2");
			return decision;
		}
		if( actual < getTargetRangeMin() ) {
			if( now == REST) {
				decision.setNextAction(HEAT);
				decision.setReasonCode("RC7.1");
			} else if(now == HEAT) {
				decision.setNextAction(HEAT);
				decision.setReasonCode("RC7.3");
			} else {
				decision.setNextAction(ACTION_ERROR);
				decision.setReasonCode("RC_ERR");
			}
			return decision;
		}
	}

	/*
	 * | RC8.1 | REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.2 | COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.3 | HEAT->HEAT because the temperature is still within target range and there is natural cooling |
	 */
	if( isTempInTargetRange(actual) ) {
		if( now == REST) {
			decision.setNextAction(REST);
			decision.setReasonCode("RC8.1");
		} else if(now == COOL) {
			decision.setNextAction(REST);
			decision.setReasonCode("RC8.2");
		} else if(now == HEAT) {
			decision.setNextAction(HEAT);
			decision.setReasonCode("RC8.3");
		} else {
			decision.setNextAction(ACTION_ERROR);
			decision.setReasonCode("RC_ERR");
		}
		return decision;
	}
	
	/*
	 * | RC9.1 | REST->COOL because even though there is natural cooling the temperature is above target range | 
	 * | RC9.2 | COOL->REST because the temperature is above target range and there is natural cooling |
	 * | RC9.3 | HEAT->REST because the temperature is above target range and there is natural cooling |
	 */
	if( actual > getTargetRangeMax() ) {
		if( now == REST) {
			decision.setNextAction(COOL);
			decision.setReasonCode("RC9.1");
		} else if(now == COOL) {
			decision.setNextAction(REST);
			decision.setReasonCode("RC9.2");
		} else if(now == HEAT) {
			decision.setNextAction(REST);
			decision.setReasonCode("RC9.3");
		} else {
			decision.setNextAction(ACTION_ERROR);
			decision.setReasonCode("RC_ERR");
		}
		return decision;
	}
		

	decision.setNextAction(ACTION_ERROR);
	decision.setReasonCode("ERROR");
	return decision;
	
}

Decision ControllerActionRules::checkFailsafeMinAndDecideAction(double actualTemp) {
	if( actualTemp < getFailsafeMin() ) {
		decision.setNextAction(HEAT);
		decision.setReasonCode("RC1");
	}
	return decision;
}

Decision ControllerActionRules::checkFailsafeMaxAndDecideAction(double actualTemp) {
	if( actualTemp > getFailsafeMax() ) {
		decision.setNextAction(COOL);
		decision.setReasonCode("RC5");
	}
	return decision;
}

Decision ControllerActionRules::checkForCoolingOverrunWithNaturalHeatingAndDecideAction(Action now, double actual) {
	if(now == COOL && actual < getStopCoolingTemp() ) { // adjust for cooling overrun
		decision.setNextAction(REST);
		decision.setReasonCode("RC2.2");
		return decision;
	}
}

boolean ControllerActionRules::isNaturalHeating(double ambient, double actual) {
	return getNaturalDrift(ambient, actual) == NATURAL_HEATING;
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

test(CheckFailsafeMin) {
	double target = 18.0;
	double range = 0.5; 
	ControllerActionRules controller(target, range);
	double ambientLow = 14.0;
	Decision decision = controller.checkFailsafeMinAndDecideAction(ambientLow);
	
	assertEqual(HEAT, decision.getNextAction());
	assertEqual("RC1", decision.getReasonCode());
}

test(CheckFailsafeMax) {
	double target = 18.0;
	double range = 0.5; 
	ControllerActionRules controller(target, range);
	double ambientHigh = 22.0;
	Decision decision = controller.checkFailsafeMaxAndDecideAction(ambientHigh);
	
	assertEqual(COOL, decision.getNextAction());
	assertEqual("RC5", decision.getReasonCode());
}

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

	double ambientLow = 14.0;
	double ambientHigh = 22.0;
	//double ambientNeutral = target;
	double belowFailsafe = 16.5;
	double belowTargetRange = 17.4;
	double withinTargetRange = target;
	double aboveTargetRange = 18.6;
	double aboveFailsafe = 19.5;
	double adjustedStopCoolingTemp = controller.getStopCoolingTemp() - 0.1;
	Action currentAction, nextAction;
	Decision decision;

	/*
	* Test 1. ambient is high, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	*/
	// Test 1.1 we are resting and ambient is high but temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	// Test 1.2 we are cooling and ambient is high but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	// Test 1.3 we are heating and ambient is high but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	/*
	 * Test 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	 */
	// Test 5.1 we are resting and ambient is high but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	// Test 5.2 we are cooling and ambient is high but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	// Test 5.3 we are heating and ambient is high but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	/*
	 * Test 2. 
	 * | RC2.1 | REST->REST because even though there is natural heating, the temperature is below the target range |
	 * | RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	 * | RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
	 */
	// Test 2.1 we are resting and ambient is high and temp is below target range so REST and use natural heating
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.1", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());

	// Test 2.2 we are cooling and ambient is high but temp is below target range so REST and use natural heating
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual(decision.getReasonCode(), "RC2.2");
	assertEqual(decision.getNextAction(), REST);
	
	// Test 2.2.1 we are cooling and ambient is high but temp is below the adjusted stop cooling temp so REST and use natural heating
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, adjustedStopCoolingTemp);
	assertEqual(decision.getReasonCode(), "RC2.2");
	assertEqual(decision.getNextAction(), REST);
	
	// Test 2.3 we are heating and ambient is high but temp is below target range so REST and use natural heating
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual(decision.getReasonCode(), "RC2.3");
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
	assertEqual(decision.getReasonCode(), "RC3.1");
	assertEqual(decision.getNextAction(), REST);
	
	// Test 3.2 we are cooling and ambient is high but temp is within target range so keep COOLing
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual(decision.getReasonCode(), "RC3.2");
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 3.3 we are heating (why!) and ambient is high but temp is within target range so REST
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual(decision.getReasonCode(), "RC3.3");
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
	assertEqual(decision.getReasonCode(), "RC4.1");
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 4.2 we are cooling and ambient is high but temp is above target range so keep COOLing
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual(decision.getReasonCode(), "RC4.2");
	assertEqual(decision.getNextAction(), COOL);
	
	// Test 4.3 we are heating (why!) and ambient is high but temp is above target range so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual(decision.getReasonCode(), "RC4.3");
	assertEqual(decision.getNextAction(), COOL);

	/*
	* Test 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	*/
	// Test 6.1 we are resting and ambient is low and temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);

	// Test 6.2 we are cooling and ambient is low but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);

	// Test 6.3 we are heating and ambient is low but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual(decision.getReasonCode(), "RC1");
	assertEqual(decision.getNextAction(), HEAT);
	
	/*
	 * | RC7.1 | REST->HEAT because the temperature is below target range and there is natural cooling |
	 * | RC7.2 | COOL->HEAT because the temperature is below the target range and there is natural cooling (adjust cooling lag?) |
	 * | RC7.3 | HEAT->HEAT because the temperature is below target range and there is natural cooling |
	 * Test 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
	 */
	// Test 7.1 we are resting and ambient is low and temp is below target range so HEAT to counteract natural cooling
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getReasonCode(), "RC7.1");
	assertEqual(decision.getNextAction(), HEAT);

	// Test 7.2 we are cooling and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getReasonCode(), "RC7.2");
	assertEqual(decision.getNextAction(), HEAT);
	
	// Test 7.2.1 we are cooling and ambient is low but temp is below below the adjusted stop cooling temp so HEAT to counteract natural cooling
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, adjustedStopCoolingTemp);
	assertEqual(decision.getReasonCode(), "RC7.2");
	assertEqual(decision.getNextAction(), HEAT);
	
	// Test 7.3 we are heating and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual(decision.getReasonCode(), "RC7.3");
	assertEqual(decision.getNextAction(), HEAT);
	
	/*
	 * | RC8.1 | REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.2 | COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.3 | HEAT->HEAT because the temperature is still within target range and there is natural cooling |
	 */	
	// Test 8.1 we are resting and ambient is low but temp is within target range so keep RESTing
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getReasonCode(), "RC8.1");
	assertEqual(decision.getNextAction(), REST);

	// Test 8.2 we are cooling and ambient is low but temp is within target range so REST
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getReasonCode(), "RC8.2");
	assertEqual(decision.getNextAction(), REST);
	
	// Test 8.3 we are heating and ambient is low but temp is within target range so keep HEATing
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual(decision.getReasonCode(), "RC8.3");
	assertEqual(decision.getNextAction(), HEAT);
	
	/*
	 * | RC9.1 | REST->COOL because even though there is natural cooling the temperature is above target range | 
	 * | RC9.2 | COOL->REST because the temperature is above target range and there is natural cooling |
	 * | RC9.3 | HEAT->REST because the temperature is above target range and there is natural cooling |
	 */
	// Test 9.1 we are resting and ambient is low but temp is above target range so keep RESTing and use natural cooling
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getReasonCode(), "RC9.1");
	assertEqual(decision.getNextAction(), COOL);

	// Test 9.2 we are cooling and ambient is low but temp is above target range so REST (and use natural cooling
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getReasonCode(), "RC9.2");
	assertEqual(decision.getNextAction(), REST);

	// Test 9.3 we are heating and ambient is low but temp is above target range so REST (and use natural cooling)
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual(decision.getReasonCode(), "RC9.3");
	assertEqual(decision.getNextAction(), REST);
	
	/*
	* Test 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	*/
	// Test 10.1 we are resting and ambient is low but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);

	// Test 10.2 we are cooling and ambient is low but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);

	// Test 10.3 we are heating and ambient is low but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.getActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual(decision.getReasonCode(), "RC5");
	assertEqual(decision.getNextAction(), COOL);
	

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

test(AdjustmentForCoolingOverrun) {
	// implement this assertEqual to test we stop cooling when we are at bottom of target range + cooling overrun adjustment.  
	// i.e. 18.0 - 0.5 + adjustment.  test all rules where we are switching cooling off but don't want to overrun the bottom of target range.  RC2.2 RC7.2
	double target = 18.0;
	double range = 0.5;
	ControllerActionRules controller(target, range);
	
	double expectedStopCoolingTemp = target - range + 0.2;
	double actualStopCoolingTemp = controller.getStopCoolingTemp();
	
	assertEqual(expectedStopCoolingTemp, actualStopCoolingTemp); 
}

test(IsNaturalHeatingOrCooling) {

	double target = 18.0;
	double range = 0.5;
	ControllerActionRules controller(target, range);

	// Test natural cooling
	double ambient = 18.0;
	double actual = 20.0;
// 	assertTrue( controller.getNaturalDrift( ambient, actual ), NATURAL_COOLING );
	
	// Test natural heating
	ambient = 18.0;
	actual = 16.0;
	assertTrue( controller.isNaturalHeating( ambient, actual ));
	
}

#endif


