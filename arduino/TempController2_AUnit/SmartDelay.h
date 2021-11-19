#ifndef _SMART_DELAY_H
#define _SMART_DELAY_H

#include <Arduino.h>

class SmartDelay {
private:
	long unsigned delayInMSec;
	long unsigned startingTimestamp;

public:
	SmartDelay(long unsigned msec);
	void start();
	long doDelay();
};

#endif
