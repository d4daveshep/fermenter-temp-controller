#line 2 "TempController2_Aunit.ino"

#include <AUnit.h>

class FermentationProfile {

  private:
    String name;
    double temp, range;
    bool valid = false;
    
  public:

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

test(fermentation_profile_get_set) {    

  String name = "TestBeer_1";
  double temp = 18.0;
  double range = 0.5;
  FermentationProfile fp1(name, temp, range);

  assertTrue(fp1.isValid());
  assertEqual(fp1.getName(), name);
  assertEqual(fp1.getFermentationTemp(), temp);
  assertEqual(fp1.getTemperatureRange(), range);
      
}

test(fermentation_temp_and_range_must_be_positive) {
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
