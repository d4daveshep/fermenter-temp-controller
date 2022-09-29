## Documentation for Python data gathering daemon ##

### Curent logic flow ###

When called as main module
* get current directory
* set up logging
* parse command line args to get config file
* call main function with config file

``main()``
* get NZ timezone for formatting timestamps
* opens the serial port via ``get_serial_port()``


``get_serial_port()``
* gets list of ``ttyACM*`` devices
* assumes the first device in the list is the one we want
* opens the serial port
* waits 30 secs for arduino to reboot - why??
* returns open serial port