# Temperature Controller Rules

### Natural heating and cooling
1. Natural heating is when the ambient temp is above the actual fermenter temp.
2. Natural cooling is when the ambient temp is below the actual fermenter temp.  

Notes:
* The larger the temperature difference the stronger the natural effect.  

### What to we do when....
#### We have natural heating...
1. Ambient is high, we are resting | cooling | heating but temp is **below failsafe.** -> *HEAT, HEAT, HEAT*
2. Ambient is high, we are resting | cooling | heating but temp is **below target range.** -> *REST, REST, REST*
3. Ambient is high, we are resting | cooling | heating but temp is **within target range.** ->  *REST, **COOL**, REST*
4. Ambient is high, we are resting | cooling | heating but temp is **above target range.** -> *COOL, COOL, COOL*
5. Ambient is high, we are resting | cooling | heating but temp is **above failsafe.** -> *COOL, COOL, COOL*
#### We have natural cooling...
6. Ambient is low, we are resting | cooling | heating but temp is **below failsafe.** -> *HEAT, HEAT, HEAT*
7. Ambient is low, we are resting | cooling | heating but temp is **below target range.** -> *HEAT, HEAT, HEAT*
8. Ambient is low, we are resting | cooling | heating but temp is **within target range.** -> *REST, REST, **HEAT***
9. Ambient is low, we are resting | cooling | heating but temp is **above target range.** -> *REST, REST, REST*
10. Ambient is low, we are resting | cooling | heating but temp is **above failsafe.** -> *COOL, COOL, COOL*

Notes:
* These rules should allow for the *HEAT to top of target range* and *COOL to bottom of target range* scenarios.  


