don't forget to add linux user to dialout group to access serial ports

crontab line for user is
* * * * * /home/pi/dev/fermenter-temp-controller/run_temp_controller.sh

crontab line for logrotate is
0 0 * * * /usr/sbin/logrotate -s /home/pi/dev/fermenter-temp-controller/logrotate.status /home/pi/dev/fermenter-temp-controller/logrotate.config

InfluxDB continuous query creation SQL syntax...
This creates a new measurement called "temp_mean_stddev" and stores the mean and stddev of "fermenter_temp" data every 5 mins

create continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"
begin
    select mean("fermenter_temp"), stddev("fermenter_temp") into "temp_mean_stddev" from "temperature" group by time(5m)
end

drop the continuous query
drop continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"





