# fermenter-temp-controller

## Dealing with temperature outliers

### Using InfluxDB continuous queries to calculate a z-score so we can detect outliers before adding them to the dataabse

Creates a new measurement called "temp_mean_stddev" and stores the mean and stddev of "fermenter_temp" data every 5 mins
````
create continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"
begin
    select mean("fermenter_temp"), stddev("fermenter_temp") into "temp_mean_stddev" from "temperature" group by time(5m)
end
````

Can't alter it once created do need to drop and re-create if changes needed
````
drop continuous query "Get_Temp_Aggregate_Data" on "99-TEST-v99"
````
The "temp_mean_stddev" contains...
````
> select * from temp_mean_stddev
name: temp_mean_stddev
time                 mean               stddev
----                 ----               ------
2021-07-10T00:35:00Z 20.203000000000007 0.01914554058097848
2021-07-10T00:40:00Z 20.138666666666666 0.0245979159943351
2021-07-10T00:45:00Z 20.076666666666657 0.02218003658672667
````

Get the last record
````
select last(*) from temp_mean_stddev
````
Returns
````
name: temp_mean_stddev
time                 last_mean          last_stddev
----                 ---------          -----------
1970-01-01T00:00:00Z 20.076666666666657 0.02218003658672667
````

### Formula to calculate a z-score:
`z = (temp - mean) / stddev` 


