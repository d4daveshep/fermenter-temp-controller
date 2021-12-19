#define _DO_UNIT_TESTING

#include <Arduino.h>
#include <AUnit.h>

#include "ControllerActionRules.h"
#include "Decision.h"


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

Decision ControllerActionRules::getDecision() {
	return this->newDecision;
}

void ControllerActionRules::resetDecision() {
	this->newDecision.clear();
}

/*
* Natural Drift happens is when ambient temp is different to actual fermenter temp.
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

Decision ControllerActionRules::getActionDecision( Action currentAction, double ambient, double actual ) {
	
	// remember newDecision is a global class variable
	newDecision.clear();

	// RC1 & RC6 | COOL because we've tripped failsafe (disregard ambient and current action)
	checkFailsafeMinAndDecideAction(actual);
	// RC5 & RC10 | HEAT because we've tripped failsafe (disregard ambient and current action)
	checkFailsafeMaxAndDecideAction(actual);

	NaturalDrift drift = getNaturalDrift(ambient, actual);
	
	// RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	// RC7.2 | COOL->HEAT because the temperature is below the target range and there is natural cooli*ng (adjust cooling lag?) |
	checkForCoolingOverrunAndDecideAction(currentAction, actual, drift);
	

	// RC2.1 | REST->REST because even though there is natural heating, the temperature is below the target range |
	// RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
	// RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
	// RC7.1 | REST->HEAT because the temperature is below target range and there is natural cooling |
	// RC7.3 | HEAT->HEAT because the temperature is below target range and there is natural cooling |
	if( actual < getTargetRangeMin() ) {
		decideActionWhenBelowTargetRange(currentAction, drift);
	}
	
	// RC3.1 | REST->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	// RC3.2 | COOL->COOL because we are still within target range and we have natural heating |
	// RC3.3 | HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise |
	// RC8.1 | REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	// RC8.2 | COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
	// RC8.3 | HEAT->HEAT because the temperature is still within target range and there is natural cooling |
	if( isTempInTargetRange(actual) ) {
		decideActionWhenInTargetRange(currentAction, drift);
	}

	// RC4.1 | REST->COOL because the temperature is above the target range and we have natural heating |
	// RC4.2 | COOL->COOL becuase the temperature is above the target range and we have natural heating |
	// RC4.3 | HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?) |
	// RC9.1 | REST->COOL because even though there is natural cooling the temperature is above target range | 
	// RC9.2 | COOL->REST because the temperature is above target range and there is natural cooling |
	// RC9.3 | HEAT->REST because the temperature is above target range and there is natural cooling |
	if( actual > getTargetRangeMax() ) {
		decideActionWhenAboveTargetRange(currentAction, drift);
	}

		
	if( newDecision.isMade() ) {
		return newDecision;
	}
	
}
	


void ControllerActionRules::checkFailsafeMinAndDecideAction(double actualTemp) {
	// remember newDecision is a global class variable
	if( !newDecision.isMade() ) {
		if( actualTemp < getFailsafeMin() ) {
			newDecision.setNextAction(HEAT);
			newDecision.setReasonCode("RC1");
		}
	}	
}

void ControllerActionRules::checkFailsafeMaxAndDecideAction(double actualTemp) {
	// remember newDecision is a global class variable
	if( !newDecision.isMade() ) {
		if( actualTemp > getFailsafeMax() ) {
			newDecision.setNextAction(COOL);
			newDecision.setReasonCode("RC5");
		}
	}
}

void ControllerActionRules::checkForCoolingOverrunAndDecideAction(Action currentAction, double actual, NaturalDrift drift) {
	// remember newDecision is a global class variable
	if( !newDecision.isMade() ) {

		if( currentAction == COOL && actual < getStopCoolingTemp() ) { // adjust for cooling overrun
			
			switch( drift) {

				case NATURAL_HEATING:
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC2.2");
					break;
				
				case NATURAL_COOLING:
					newDecision.setNextAction(HEAT);
					newDecision.setReasonCode("RC7.2");
					break;
				
				default:
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
					break;
			}				
		}
	}
}

void ControllerActionRules::decideActionWhenBelowTargetRange(Action currentAction, NaturalDrift drift) {
	if (!newDecision.isMade() ) {
		switch( drift ) {
			case NATURAL_HEATING:
				if( currentAction == REST) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC2.1");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC2.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}
				break;

			case NATURAL_COOLING:
				if( currentAction == REST) {
					newDecision.setNextAction(HEAT);
					newDecision.setReasonCode("RC7.1");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(HEAT);
					newDecision.setReasonCode("RC7.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}
				break;
			
			default:
				newDecision.setNextAction(ACTION_ERROR);
				newDecision.setReasonCode("RC_ERR");
				break;
		}
	}
}

void ControllerActionRules::decideActionWhenInTargetRange(Action currentAction, NaturalDrift drift) {
	if(!newDecision.isMade()) {
		switch( drift ) {
			case NATURAL_HEATING:
				if( currentAction == REST) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC3.1");
				} else if(currentAction == COOL) {
					newDecision.setNextAction(COOL);
					newDecision.setReasonCode("RC3.2");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC3.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}	
				break;
				
			case NATURAL_COOLING:
				if( currentAction == REST) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC8.1");
				} else if(currentAction == COOL) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC8.2");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(HEAT);
					newDecision.setReasonCode("RC8.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}
				break;
				
			default:
				newDecision.setNextAction(ACTION_ERROR);
				newDecision.setReasonCode("RC_ERR");
				break;
		}
	}
}
void ControllerActionRules::decideActionWhenAboveTargetRange(Action currentAction, NaturalDrift drift) {
	if(!newDecision.isMade()) {
		switch( drift ) {
			case NATURAL_HEATING:
				if( currentAction == REST) {
					newDecision.setNextAction(COOL);
					newDecision.setReasonCode("RC4.1");
				} else if(currentAction == COOL) {
					newDecision.setNextAction(COOL);
					newDecision.setReasonCode("RC4.2");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(COOL);
					newDecision.setReasonCode("RC4.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}
				break;
				
			case NATURAL_COOLING:
				if( currentAction == REST) {
					newDecision.setNextAction(COOL);
					newDecision.setReasonCode("RC9.1");
				} else if(currentAction == COOL) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC9.2");
				} else if(currentAction == HEAT) {
					newDecision.setNextAction(REST);
					newDecision.setReasonCode("RC9.3");
				} else {
					newDecision.setNextAction(ACTION_ERROR);
					newDecision.setReasonCode("RC_ERR");
				}
				break;
			
			default:
				newDecision.setNextAction(ACTION_ERROR);
				newDecision.setReasonCode("RC_ERR");
				break;
		}
	}
}


