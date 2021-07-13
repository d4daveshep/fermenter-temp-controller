# fermenter-temp-controller

## Calculating "heating lag"
Heating lag is when we tell the temperature controller to stop or start heating but it takes a while before hte temperature changes direction
Psuedo code...
````
temp_range_min = target_temp - temp_diff  # eg 20.0 C - 0.2 C = 19.8 C
heat_start_lag = abs( heat_start_temp - min_temp_after_heat_start )  # e.g. 19.8 - 19.75 = 0.05
heat_start_temp = temp_range_min + heat_start_lag

So start heating when temp < heat_start_temp

temp_range_max = target_temp + temp_diff  # eg 20.0 C + 0.2 C = 20.2 C
heat_stop_lag = abs( heat_stop_lag - max_temp_after_heat_stop )
heat_stop_temp = temp_max_range - heat_stop_lag

So stop heating when temp > heat_stop_temp 
````



## Dealing with temperature outliers
### Using InfluxDB continuous queries to calculate a z-score so we can detect outliers before adding them to the dataabse

Creates a new measurement called "temp_mean_stddev" and stores the mean and stddev of "fermenter_temp" data every 10 mins
````buildoutcfg
create continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"
begin
    select mean("fermenter_temp"), stddev("fermenter_temp") into "temp_mean_stddev" from "temperature" group by time(10m)
end
````

Can't alter it once created do need to drop and re-create if changes needed
````buildoutcfg
drop continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"
````
The "temp_mean_stddev" contains...
````buildoutcfg
> select * from temp_mean_stddev
name: temp_mean_stddev
time                 mean               stddev
----                 ----               ------
2021-07-10T00:35:00Z 20.203000000000007 0.01914554058097848
2021-07-10T00:40:00Z 20.138666666666666 0.0245979159943351
2021-07-10T00:45:00Z 20.076666666666657 0.02218003658672667
````

Get the last record
````buildoutcfg
select last(*) from temp_mean_stddev
````
Returns
````buildoutcfg
name: temp_mean_stddev
time                 last_mean          last_stddev
----                 ---------          -----------
1970-01-01T00:00:00Z 20.076666666666657 0.02218003658672667
````

### Formula to calculate a z-score:
````buildoutcfg
z = (temp - mean) / stddev
```` 

### Psuedo code to check and omit an outlier
````buildoutcfg
1. get the fermenter_temp
2. get the last values from the temp_mean_stddev table
3. calculate the z-score
4. if z-score < some tolerance e.g. 3.0 then
    4.1. write the temp record to database
    else
    4.2. don't write the temp record to database
````


