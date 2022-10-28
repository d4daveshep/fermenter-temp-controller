
## Documentation for Python data gathering daemon ##

### Logic flow after refactoring ###
#### Initialisation ####
``config.py`` module defines ``ControllorConfig`` class
* [x] reads and validates the specified config file
* [x] sets config settings in object attributes fermenter, timezone and influxdb credentials
* [x] open the database connection
* [x] open the serial port
* [x] write ``target_temp`` to serial port

#### Main loop ####
- [ ] ``while(true)`` loop
* check for messages from the web api server - i.e. change target temp, change brew ID
* [x] send any messages (e.g. new target temp) to the Arduino controller via serial port
* [x] read the arduino state (current action, temperature data) json string from Arduino controller via serial port
* get current timestamp
* [x] write data to database with timestamp

#### Messages from web API ####
* get current target temp and brew ID
* set new target temp
* set new brew ID
* exit/stop/kill (not sure if this is possible)


  

### Logic flow before refactoring ###

When called as main module
* get current directory
* set up logging
* parse command line args to get config file
* call main function with config file

``main()``
* get NZ timezone for formatting timestamps
* opens the serial port via ``get_serial_port()``
* reads the config file with ``ConfigParser``
* gets ``brew_id`` from the config file
* gets the target temperature from the config file
* encodes and writes the target temp to the serial port
* opens the influxdb client via ``open_influxdb()`` passing ``brew_id`` as the database name
* creates a dictionary object for the influxdb record to write
* enters a infinite while loop
  * reads a line from the serial port
  * converts it to a json object
  * gets a current timestamp
  * copies data from the json object to the influxdb dictionary object
  * checks if the target temp has changed (why would it?)
  * writes a new target temp to the serial port if needed (not sure if this code ever gets called or works)
  * checks if the temperature values are outliers via a z_score calculation and comparision to the recent mean, sttdev (not sure I need to bother with this anymore)
  * writes the dictionary object to the influxdb database
* closes the database if the while loop exits (would we ever get to these lines?)


``get_serial_port()``
* gets list of ``ttyACM*`` devices
* assumes the first device in the list is the one we want
* opens the serial port
* waits 30 secs for arduino to reboot - why??
* returns open serial port

``open_influxdb()``
* uses hardcoded host and port
* creates the database (``brew_id``) if it doesn't exist already
* sets a retention policy of 4 weeks
* creates a continuous query for mean and stddev of last 10 mins of temp readings
* returns the client object

``get_last_mean_stddev()``
* runs a query against the continuous query set up in influxdb to get last mean and sttdev values for 10 mins
* returns the values 
