#line 2 "TempController2_Aunit.ino"

#define DO_UNIT_TESTING

#ifdef DO_UNIT_TESTING
#include <AUnit.h>
#endif

#include "MyClass.h"

MyClass me;
String name = me.getName();

class FermentationProfile {

  private:
    String name;
    double temp, range;
    bool valid = false;
    
  public:

    FermentationProfile() {
    }
    
    FermentationProfile(String name, double temp, double range){
      this->name = name;

      if( temp > 0.0 && range > 0.0) {
        this->valid = true;
      }
      this->temp = temp;
      this->range = range;
    }

    String getName() {
      return this->name;
    }

    double getFermentationTemp() {
      return this->temp;
    }

    double setFermentationTemp(double new_temp) {
      this->temp = new_temp;
    }

    double getTemperatureRange() {
      return this->range;
    }

    bool isValid() {
      return this->valid;
    }
};

enum Action { ACTION_ERROR, HEAT, COOL, REST };
enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING };

class ControllerActionRules {

  private:
  FermentationProfile profile;
  double ambientDriftThreshhold = 1.0; // difference between ambient and target that we assume will be influencial

  public:
  ControllerActionRules(FermentationProfile fp) {
    this->profile = fp;
  }

  double getFailsafeMin() {
    return profile.getFermentationTemp() - ( profile.getTemperatureRange() * 2 );
  }

  double getFailsafeMax() {
    return profile.getFermentationTemp() + ( profile.getTemperatureRange() * 2 );
  }

  double getTargetRangeMin() {
    return profile.getFermentationTemp() - profile.getTemperatureRange();
  }

  double getTargetRangeMax() {
    return profile.getFermentationTemp() + profile.getTemperatureRange();
  }

  bool inTargetRange(double temp) {
    return getTargetRangeMin() <= temp && temp <= getTargetRangeMax();
  }
  

  /*
   * Natural Drift hapens is when ambient temp is different to actual fermenter temp.
   * When ambient > actual we have natural heating
   * When ambient < actual we have natural cooling
   * Note:  target temp is irrelevant
   */
  NaturalDrift getNaturalDrift(double ambient, double actual) {

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

  Action getNextAction( Action now, double ambient, double actual ) {
    
    // Test 1,6. if we've tripped failsafe then disregard ambient and current action
    if( actual < getFailsafeMin() ) {
      return HEAT;
    }

    // Test 5,10. if we've tripped failsafe then disregard ambient and current action
    if( actual > getFailsafeMax() ) {
      return COOL;
    }

    // Test 2. regardless of what we're currently doing, we are above our target range and have natural heating so start cooling
    if(actual > getTargetRangeMax() && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
      return COOL;
    }

    // Test 4. Regardless of what we're doing, when ambient is high and we are below target range, just REST and use natural heating
    if( actual < getTargetRangeMin() && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
      return REST;
    }

    // Test 7.  Regardless of what we're currently doing, we are below our target range and have natural cooling so start heating
    if(actual < getTargetRangeMin() && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
      return HEAT;
    }

    // Test 9. Regardless of what we're doing, when ambient is low, but temp is above target range, we REST and use natural cooling
    if(actual > getTargetRangeMax() && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
      return REST;
    }

    
    
    // Test 3.1 we're already resting within our target range and have natural heating so keep resting
    if( now == REST && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
      return REST;
    }
    
    // Test 3.2 we're already cooling within our target range and have natural heating so keep cooling
    if( now == COOL && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
      return COOL;
    }
    
    // Test 3.3 we're heating within our target range and have natural heating so stop and rest
    if( now == HEAT && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_HEATING ) {
      return REST;
    }

    // Test 8.1 we are resting within our target range and ambient is low so keep RESTing
    if( now == REST && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
      return REST;
    }

    // Test 8.2 we are cooling, temp is within target range and ambient is low so REST (and use natural cooling)
    if( now == COOL && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
      return REST;
    }
  
    // Test 8.3 we are heating. temp is within target range and ambient is low so keep HEATing
    if( now == HEAT && inTargetRange(actual) && getNaturalDrift(ambient, actual) == NATURAL_COOLING ) {
      return HEAT;
    }
  

    return ACTION_ERROR;
  }

};
  
#ifdef DO_UNIT_TESTING

// Test the decision making logic
test(WhatToDoNext) {
  String name = "TestBeer_1";
  double target = 18.0;
  double range = 0.5; 
  FermentationProfile fp1(name, target, range);
  ControllerActionRules controller(fp1);

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

  /*
   * Test 1. ambient is high, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
   */
  // Test 1.1 we are resting and ambient is high but temp is below failsafe so HEAT
  currentAction = REST;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowFailsafe);
  assertEqual(nextAction, HEAT);

  // Test 1.2 we are cooling and ambient is high but temp is below failsafe so HEAT
  currentAction = COOL;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowFailsafe);
  assertEqual(nextAction, HEAT);

  // Test 1.3 we are heating and ambient is high but temp is below failsafe so HEAT
  currentAction = HEAT;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowFailsafe);
  assertEqual(nextAction, HEAT);

  /*
   * Test 2. ambient is high, we are resting | cooling | heating but temp is below target range.  REST, REST, REST
   */
  // Test 2.1 we are resting and ambient is high and temp is below target range so REST and use natural heating
  currentAction = REST;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
  assertEqual(nextAction, REST);

  // Test 2.2 we are cooling and ambient is high but temp is below target range so REST and use natural heating
  currentAction = COOL;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
  assertEqual(nextAction, REST);

  // Test 2.3 we are heating and ambient is high but temp is below target range so REST and use natural heating
  currentAction = HEAT;
  nextAction = controller.getNextAction(currentAction, ambientHigh, belowTargetRange);
  assertEqual(nextAction, REST);

  /*
   * Test 3. ambient is high, we are resting | cooling | heating but temp is within target range.  REST, COOL, REST
   */
  // Test 3.1 we are resting and ambient is high but temp is within target range so keep RESTing
  currentAction = REST;
  nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
  assertEqual(nextAction, REST);

  // Test 3.2 we are cooling and ambient is high but temp is within target range so keep COOLing
  currentAction = COOL;
  nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
  assertEqual(nextAction, COOL);

  // Test 3.3 we are heating (why!) and ambient is high but temp is within target range so REST
  currentAction = HEAT;
  nextAction = controller.getNextAction(currentAction, ambientHigh, withinTargetRange);
  assertEqual(nextAction, REST);

  /*
   * Test 4. ambient is high, we are resting | cooling | heating but temp is above target range.  COOL, COOL, COOL
   */
  // Test 4.1 we are resting and ambient is high but temp is above target range so start COOLing
  currentAction = REST;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
  assertEqual(nextAction, COOL);

  // Test 4.2 we are cooling and ambient is high but temp is above target range so keep COOLing
  currentAction = COOL;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
  assertEqual(nextAction, COOL);

  // Test 4.3 we are heating (why!) and ambient is high but temp is above target range so COOL
  currentAction = HEAT;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveTargetRange);
  assertEqual(nextAction, COOL);


  /*
   * Test 5. ambient is high, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
   */
  // Test 5.1 we are resting and ambient is high but temp is above failsafe so COOL
  currentAction = REST;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
  assertEqual(nextAction, COOL);

  // Test 5.2 we are cooling and ambient is high but temp is above failsafe so COOL
  currentAction = COOL;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
  assertEqual(nextAction, COOL);

  // Test 5.3 we are heating and ambient is high but temp is above failsafe so COOL
  currentAction = HEAT;
  nextAction = controller.getNextAction(currentAction, ambientHigh, aboveFailsafe);
  assertEqual(nextAction, COOL);

  /*
   * Test 6. ambient is low, we are resting | cooling | heating but temp is below failsafe.  HEAT, HEAT, HEAT
   */
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

  /*
   * Test 7. ambient is low, we are resting | cooling | heating but temp is below target range. HEAT, HEAT, HEAT 
   */
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

  /*
   * Test 8. ambient is low, we are resting | cooling | heating but temp is within target range. REST, REST, HEAT
   */
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

  /*
   * Test 9. ambient is low, we are resting | cooling | heating but temp is above target range. REST, REST, REST 
   */
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

  /*
   * Test 10. ambient is low, we are resting | cooling | heating but temp is above failsafe. COOL, COOL, COOL
   */
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



  
  //assertTrue(false); // finishing these tests


  
}

test(AmbientTempGivesNaturalCoolingOrHeating) {

  String name = "TestBeer_1";
  double target = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, target, range);
  ControllerActionRules controller(fp1);

  
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

test(TempAndRangeMustBePositive) {
    FermentationProfile fp1("TestBeer_1", 18.0, -1); // this should fail
    assertFalse(fp1.isValid());
    FermentationProfile fp2("TestBeer_2", -1, 1.0 ); // sets to invalid
    assertFalse(fp2.isValid());
   
}

#endif

//----------------------------------------------------------------------------
// setup() and loop()
//----------------------------------------------------------------------------

void setup() {
  delay(1000); // wait for stability on some boards to prevent garbage Serial
  Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
  while(!Serial); // for the Arduino Leonardo/Micro only

  Serial.print("\n");

}

void loop() {
#ifdef DO_UNIT_TESTING
  aunit::TestRunner::run();
#endif
}
