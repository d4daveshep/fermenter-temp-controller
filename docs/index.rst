don't forget to add linux user to dialout group to access serial ports

crontab line for user is
* * * * * /home/pi/dev/fermenter-temp-controller/run_temp_controller.sh

crontab line for logrotate is
0 0 * * * /usr/sbin/logrotate -s /home/pi/dev/fermenter-temp-controller/logrotate.status /home/pi/dev/fermenter-temp-controller/logrotate.config







