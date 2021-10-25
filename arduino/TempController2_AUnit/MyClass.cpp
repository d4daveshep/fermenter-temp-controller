#include <Arduino.h>
#include <AUnit.h>
#include "MyClass.h"

MyClass::MyClass() {}

String MyClass::getName() {
	return "Dave";
}

#ifdef DO_UNIT_TESTING
test(DoIt) {
	MyClass me;
	assertEqual(me.getName(), "Dave");
}
#endif
