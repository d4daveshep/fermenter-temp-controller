#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "ControllerActionRules.h"
#include "Decision.h"

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

test(UpdatedTargetTempIsSaved) {
	double defaultTargetTemp = 6.0;
	double defaultRange = 0.3; // i.e. +/- either side of target
	ControllerActionRules controller(defaultTargetTemp, defaultRange);
	
	double newTargetTemp = 7.0;
	controller.setTargetTemp(newTargetTemp);
	
	assertEqual(controller.getTargetTemp(), newTargetTemp );
}

test(CheckFailsafeMinAndResetDecision) {
	double target = 18.0;
	double range = 0.5; 
	ControllerActionRules controller(target, range);
	double ambientLow = 14.0;
	
	controller.resetDecision();
	controller.checkFailsafeMinAndDecideAction(ambientLow);
	Decision decision = controller.getDecision();
	
	assertEqual(HEAT, decision.getNextAction());
	assertEqual("RC1", decision.getReasonCode());
	
	controller.resetDecision();
	decision = controller.getDecision();
	assertFalse( decision.isMade() );
}

test(CheckFailsafeMax) {
	double target = 18.0;
	double range = 0.5; 
	ControllerActionRules controller(target, range);
	double ambientHigh = 22.0;
	
	controller.resetDecision();
	controller.checkFailsafeMaxAndDecideAction(ambientHigh);
	Decision decision = controller.getDecision();
	
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
	controller.resetDecision();
	// Test 1.1 we are resting and ambient is high but temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 1.2 we are cooling and ambient is high but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 1.3 we are heating and ambient is high but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	/*
	 * Test 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	 */
	controller.resetDecision();
	// Test 5.1 we are resting and ambient is high but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 5.2 we are cooling and ambient is high but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 5.3 we are heating and ambient is high but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	/*
	 * Test 2. 
	 * | RC2.1 | REST->REST because even though there is natural heating, the temperature is below the target range |
	 * | RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	 * | RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
	 */
	controller.resetDecision();
	// 	Test 2.1 we are resting and ambient is high and temp is below target range so REST and use natural heating
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.1", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());

	controller.resetDecision();
	// 	Test 2.2 we are cooling and ambient is high but temp is below target range so REST and use natural heating
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.2", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	controller.resetDecision();
	// 	Test 2.2.1 we are cooling and ambient is high but temp is below the adjusted stop cooling temp so REST and use natural heating
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, adjustedStopCoolingTemp);
	assertEqual("RC2.2", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	controller.resetDecision();
	// 	Test 2.3 we are heating and ambient is high but temp is below target range so REST and use natural heating
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientHigh, belowTargetRange);
	assertEqual("RC2.3", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	/*
	* Test 3. ambient is high, we are resting | cooling | heating but temp is within target range.  REST, COOL, REST
	* | RC3.1 | REST->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	* | RC3.2 | COOL->COOL because we are still within target range and we have natural heating |
	* | RC3.3 | HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	* 
	*/
	controller.resetDecision();
	// Test 3.1 we are resting and ambient is high but temp is within target range so keep RESTing
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.1", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	controller.resetDecision();
	// Test 3.2 we are cooling and ambient is high but temp is within target range so keep COOLing
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.2", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 3.3 we are heating (why!) and ambient is high but temp is within target range so REST
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientHigh, withinTargetRange);
	assertEqual("RC3.3", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	/*
	* Test 4. ambient is high, we are resting | cooling | heating but temp is above target range.  COOL, COOL, COOL
	* | RC4.1 | REST->COOL because the temperature is above the target range and we have natural heating |
	* | RC4.2 | COOL->COOL becuase the temperature is above the target range and we have natural heating |
	* | RC4.3 | HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?) |
	*/
	controller.resetDecision();
	// 	Test 4.1 we are resting and ambient is high but temp is above target range so start COOLing
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.1", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 4.2 we are cooling and ambient is high but temp is above target range so keep COOLing
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.2", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 4.3 we are heating (why!) and ambient is high but temp is above target range so COOL
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientHigh, aboveTargetRange);
	assertEqual("RC4.3", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());

	/*
	* Test 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
	*/
	controller.resetDecision();
	// Test 6.1 we are resting and ambient is low and temp is below failsafe so HEAT
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());

	controller.resetDecision();
	// Test 6.2 we are cooling and ambient is low but temp is below failsafe so HEAT
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 6.3 we are heating and ambient is low but temp is below failsafe so HEAT
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowFailsafe);
	assertEqual("RC1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	/*
	 * | RC7.1 | REST->HEAT because the temperature is below target range and there is natural cooling |
	 * | RC7.2 | COOL->HEAT because the temperature is below the target range and there is natural cooling (adjust cooling lag?) |
	 * | RC7.3 | HEAT->HEAT because the temperature is below target range and there is natural cooling |
	 * Test 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
	 */
	controller.resetDecision();
	// Test 7.1 we are resting and ambient is low and temp is below target range so HEAT to counteract natural cooling
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual("RC7.1", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 7.2 we are cooling and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual("RC7.2", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 7.2.1 we are cooling and ambient is low but temp is below below the adjusted stop cooling temp so HEAT to counteract natural cooling
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, adjustedStopCoolingTemp);
	assertEqual("RC7.2", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	controller.resetDecision();
	// Test 7.3 we are heating and ambient is low but temp is below target range so HEAT to counteract natural cooling
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientLow, belowTargetRange);
	assertEqual("RC7.3", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	/*
	 * | RC8.1 | REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.2 | COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	 * | RC8.3 | HEAT->HEAT because the temperature is still within target range and there is natural cooling |
	 */	
	controller.resetDecision();
	// Test 8.1 we are resting and ambient is low but temp is within target range so keep RESTing
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual("RC8.1", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());

	controller.resetDecision();
	// Test 8.2 we are cooling and ambient is low but temp is within target range so REST
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual("RC8.2", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	controller.resetDecision();
	// Test 8.3 we are heating and ambient is low but temp is within target range so keep HEATing
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientLow, withinTargetRange);
	assertEqual("RC8.3", decision.getReasonCode());
	assertEqual(HEAT, decision.getNextAction());
	
	/*
	 * | RC9.1 | REST->REST because the temperature is above target range and there is natural cooling |
	 * | RC9.2 | COOL->REST because the temperature is above target range and there is natural cooling |
	 * | RC9.3 | HEAT->REST because the temperature is above target range and there is natural cooling |
	 */
	controller.resetDecision();
	// Test 9.1 we are resting and ambient is low but temp is above target range so keep RESTing and use natural cooling
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual("RC9.1", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());

	controller.resetDecision();
	// Test 9.2 we are cooling and ambient is low but temp is above target range so keep COOLing 
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual("RC9.2", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());

	controller.resetDecision();
	// Test 9.3 we are heating and ambient is low but temp is above target range so REST (and use natural cooling)
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveTargetRange);
	assertEqual("RC9.3", decision.getReasonCode());
	assertEqual(REST, decision.getNextAction());
	
	/*
	* Test 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
	*/
	controller.resetDecision();
	// Test 10.1 we are resting and ambient is low but temp is above failsafe so COOL
	currentAction = REST;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());

	controller.resetDecision();
	// Test 10.2 we are cooling and ambient is low but temp is above failsafe so COOL
	currentAction = COOL;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	
	controller.resetDecision();
	// Test 10.3 we are heating and ambient is low but temp is above failsafe so COOL
	currentAction = HEAT;
	decision = controller.makeActionDecision(currentAction, ambientLow, aboveFailsafe);
	assertEqual("RC5", decision.getReasonCode());
	assertEqual(COOL, decision.getNextAction());
	

// 	assertTrue(false); // finishing these tests

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

#endif


