# Overall Temperature Controller Flow Logic

### Setup Loop
* Opens the serial port
* Sets up the relay pins
* Sets up the LCD
* Initialises the actual temp and ambient readings
* Initialises some readings for attempt at ML
* Initialises timestamps  

### Main Loop
* Reads the serial port and sets the target temp if updated
* Reads actual and ambient temp sensors
Decides what to do from temp control rules
* Checks overrides / failsafe
* Checks if we're within target range or not
* Sets current (i.e. next) action (which i'm calling next action)
Carries out the action
* Updates the relay state
* Updates the LCD display
* Prints JSON output to serial port for logging
* Does a smart delay of 1 sec

### Fuctions defined - these rely on lots of global variables 
* `printJSON()` to Serial
* `debug` prints similar info to Serial as printJSON()
* `randomTemp()` returns random temp value between 15 - 25
* `readSerialWithStartEndMarkers()` into `receivedChars[]`
* `updateTargetTemp()` using `receivedChars[]`
* `doTempReadings()` and calculate average of last 10 actual temp readings, also update min and max actual tempss
* `updateLCD()` to dispay actual, target, ambient, action, min & max actual
* `simCurrentTemp()` and `simAmbientTemp() does some random temperature simulations 


