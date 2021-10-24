#line 2 "TempController2_Aunit.ino"

#include <AUnit.h>

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

    double getTemperatureRange() {
      return this->range;
    }

    bool isValid() {
      return this->valid;
    }
};

enum Action { ACTION_ERROR, HEAT, COOL, REST };
enum NaturalDrift { DRIFT_ERROR, NATURAL_HEATING, NATURAL_COOLING, NEUTRAL };

class ControllerActionRules {

  private:
  FermentationProfile profile;
  double ambientDriftThreshhold = 1.0; // difference between ambient and target that we assume will be influencial

  public:
  ControllerActionRules() {
  }
  ControllerActionRules(FermentationProfile fp) {
    this->profile = fp;
  }
  
  Action getAction(double actualTemp) {

    double target = profile.getFermentationTemp();
    double range = profile.getTemperatureRange();
    Action action = ACTION_ERROR;  // set error if we can't make a decision
    //Serial.print(actualTemp);
    //Serial.print("\n");
  
    // check failsafe is not breached
    if( actualTemp > (target + range*2) ) {
      action = COOL;
      return action;
    }
    if (actualTemp < (target - range*2) ) {
      action = HEAT;
      return action;
    }
    
    // temp above range
    if( actualTemp > (target + range )) {
      action = COOL;
      return action;
    }
  
    // temp below range
    if( actualTemp < target - range ) {
      action = HEAT;
      return action;
    }
    
    // temp within range
    if( actualTemp >= (target - range) && actualTemp <= (target + range) ) {
      action = REST;
      return action;
    }
    
    return action;
    
  }


  NaturalDrift getNaturalDrift(double ambient) {
    double target = profile.getFermentationTemp();
    NaturalDrift drift = DRIFT_ERROR;

    // if ambient is 1C below target then assume natural cooling to heat to top of range
    if(ambient < (target - this->ambientDriftThreshhold)) {
      drift = NATURAL_COOLING;
      return drift;
    }

    // if ambient is 1C above target then assume natural heating and cool to bottom of range
    if(ambient > (target + this->ambientDriftThreshhold)) {
      drift = NATURAL_HEATING;
      return drift;
    }

    // if ambient is within 1C of target then call this a neutral environment
    if(ambient >= (target - this->ambientDriftThreshhold) && ambient <= (target + this->ambientDriftThreshhold)) {
      drift = NEUTRAL;
      return drift;
    }
   
    return DRIFT_ERROR;
  }

};
  

// Test the decision making logic


test(AmbientTempGivesNaturalCoolingOrHeating) {

  String name = "TestBeer_1";
  double target = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, target, range);
  ControllerActionRules controller(fp1);

  /* 
   *  When ambient temp is "well below" target range then don't need much active cooling (except failsafe)
   *  So we should heat or cool to the TOP of the range and allow natural cooling to do the rest
   */
   
  double ambient = target - 1.1; // set below target by > 1C
  NaturalDrift drift = controller.getNaturalDrift(ambient);
  assertEqual(drift, NATURAL_COOLING);
  
  /* 
   *  When ambient temp is "well above" target range then don't need much active heating (except failsafe)
   *  So we should heat or cool to the BOTTOM of the range and allow natural heating to do the rest
   */
   
  ambient = target + 1.1; // set above target by > 1C
  drift = controller.getNaturalDrift(ambient);
  assertEqual(drift, NATURAL_HEATING);
  
  /* 
   *  When ambient temp is close to target range then will rely on active heat and cooling
   *  So we should heat or cool to the BOTTOM of the range and allow natural heating to do the rest
   */
   
  ambient = target;
  drift = controller.getNaturalDrift(ambient);
  assertEqual(drift, NEUTRAL);
  
  //assertTrue(false); // deliberately fail this test 
  
}

test(TempRangeExceeded) {
  String name = "TestBeer_1";
  double temp = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, temp, range);
  ControllerActionRules controller(fp1);

  Action action;
  action = controller.getAction(temp + range*1.1);  // just over range - should cool
  assertEqual(action, COOL);

  action = controller.getAction(temp - range*1.1);  // just under range - should heat
  assertEqual(action, HEAT);

  action = controller.getAction(temp + range*0.9);  // just under top end of range. should rest
  assertEqual(action, REST);
  
  action = controller.getAction(temp - range*0.9);  // just above bottom end of range. should rest
  assertEqual(action, REST);
  
}

test(FailsafeExceeded) {
  String name = "TestBeer_1";
  double temp = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, temp, range);
  ControllerActionRules controller(fp1);

  Action action;
  action = controller.getAction(temp+range*3);  // way above range so cool
  assertEqual(action, COOL);

  action = controller.getAction(temp-range*3);  // way below range so heat
  assertEqual(action, HEAT);

}


/*
test(fermentation_profile_get_set) {    
  String name = "TestBeer_1";
  double temp = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, temp, range);
  assertTrue(fp1.isValid());
  assertEqual(fp1.getName(), name); // not really needed
  assertEqual(fp1.gFermentationProfile profile

etFermentationTemp(), temp);
  assertEqual(fp1.getTemperatureRange(), range);
       FermentationProfile fp1("TestBeer_1", 18.0, 0.5);

}
*/
test(TempAndRangeMustBePositive) {
    FermentationProfile fp1("TestBeer_1", 18.0, -1); // this should fail
    assertFalse(fp1.isValid());
    FermentationProfile fp2("TestBeer_2", -1, 1.0 ); // sets to invalid
    assertFalse(fp2.isValid());
   
}


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
  aunit::TestRunner::run();
}
