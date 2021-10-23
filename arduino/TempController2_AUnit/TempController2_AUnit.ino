#line 2 "TempController2_Aunit.ino"

#include <AUnit.h>

class FermentationProfile {

  public:

    FermentationProfile(String name){
    
    }

    String getName() {
      return "";
    }
};

test(fermentation_profile_get_set) {
  FermentationProfile fp1("MyBeer");
  assertEqual(fp1.getName(), "MyBeer");
}

test(correct) {
  int x = 1;
  assertEqual(x, 1);
}

test(incorrect) {
  int x = 1;
  assertNotEqual(x, 1);
}

//----------------------------------------------------------------------------
// setup() and loop()
//----------------------------------------------------------------------------

void setup() {
  delay(1000); // wait for stability on some boards to prevent garbage Serial
  Serial.begin(115200); // ESP8266 default of 74880 not supported on Linux
  while(!Serial); // for the Arduino Leonardo/Micro only

}

void loop() {
  aunit::TestRunner::run();
}
