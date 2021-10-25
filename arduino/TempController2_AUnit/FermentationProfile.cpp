#include <Arduino.h>
#include <AUnit.h>
#include "FermentationProfile.h"

FermentationProfile::FermentationProfile() {}

FermentationProfile::FermentationProfile(String name, double temp, double range){
	this->name = name;

	if( temp > 0.0 && range > 0.0) {
		this->valid = true;
	}
	this->temp = temp;
	this->range = range;
}

String FermentationProfile::getName() {
	return this->name;
}

double FermentationProfile::getFermentationTemp() {
	return this->temp;
}

double FermentationProfile::setFermentationTemp(double new_temp) {
	this->temp = new_temp;
}

double FermentationProfile::getTemperatureRange() {
	return this->range;
}

bool FermentationProfile::isValid() {
	return this->valid;
}

/*
 * AUnit Tests
 */
test(TempAndRangeMustBePositive) {
    FermentationProfile fp1("TestBeer_1", 18.0, -1); // this should fail
    assertFalse(fp1.isValid());
    FermentationProfile fp2("TestBeer_2", -1, 1.0 ); // sets to invalid
    assertFalse(fp2.isValid());
   
}

