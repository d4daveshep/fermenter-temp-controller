# Temperature Controller Rules #  
### Easy Rules ###  

#### Natural heating and cooling ####
1. Natural heating is when the ambient temp is above the actual fermenter temp.
2. Natural cooling is when the ambient temp is below the actual fermenter temp.  

Notes:
* The larger the temperature difference the stronger the natural effect.  

### Natural Heating Rules ###
#### Failsafe Exceeded ####
1. If we are outside target range by a lot then trigger a **failsafe**, and HEAT or COOL until we are back within the target range.  

#### Outside of Target Range ####  
1. If we are below the target range AND have natural heating, then HEAT to bottom of target range then REST to naturally warm up.  
1. If we are above the target range AND have natural heating, then COOL to near bottom of target range 

#### Within Target Range ####
1. If we are within target range and COOLING then keep COOLIING to near bottom of target range then REST to naturally warm up.


2. If we are below the target range AND have natural cooling, then HEAT to near top of target range then REST to naturally cool down.
4. If we are above the target range AND have natural cooling, then COOL to top of target range then REST to naturally cool down

THIS IS REALLY HARD!!!  
Should I try to remember and stick to the previous strategy e.g. heat to top of target range.  Or should I just heat until another rule takes over?

