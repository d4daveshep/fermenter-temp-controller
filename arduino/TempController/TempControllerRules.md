# Temperature Controller Rules

### Natural heating and cooling
1. Natural heating is when the ambient temp is above the actual fermenter temp.
2. Natural cooling is when the ambient temp is below the actual fermenter temp.  

Notes:
* The larger the temperature difference the stronger the natural effect.  

### What to we do when....
#### We have natural heating...
1. Ambient is high, we are resting | cooling | heating but temp is **below failsafe.** -> *HEAT, HEAT, HEAT*
2. Ambient is high, we are resting | cooling | heating but temp is **below target range.** -> ***HEAT**, REST, REST*
3. Ambient is high, we are resting | cooling | heating but temp is **within target range.** ->  *REST, **COOL**, REST*
4. Ambient is high, we are resting | cooling | heating but temp is **above target range.** -> *COOL, COOL, COOL*
5. Ambient is high, we are resting | cooling | heating but temp is **above failsafe.** -> *COOL, COOL, COOL*
#### We have natural cooling...
6. Ambient is low, we are resting | cooling | heating but temp is **below failsafe.** -> *HEAT, HEAT, HEAT*
7. Ambient is low, we are resting | cooling | heating but temp is **below target range.** -> *HEAT, HEAT, HEAT*
8. Ambient is low, we are resting | cooling | heating but temp is **within target range.** -> *REST, REST, **HEAT***
9. Ambient is low, we are resting | cooling | heating but temp is **above target range.** -> ***COOL**, REST, REST*
10. Ambient is low, we are resting | cooling | heating but temp is **above failsafe.** -> *COOL, COOL, COOL*

Notes:
* These rules should allow for the *HEAT to top of target range* and *COOL to bottom of target range* scenarios.  

### Reason Codes
#### Failsafe range exceeded
| Code | Reason |
| ---- | ------ |
| RC1, RC6 | HEAT because the temperature is below the failsafe minimum, regardless of ambient conditions |
| RC5, RC10 | COOL because the temperature is above the failsafe maximum, regardless of ambient conditions |

#### With natural heating
| Code | Reason |
| ---- | ------ |
| RC2.1 | REST->HEAT because even though there is natural heating, the temperature is below the target range |
| RC2.2 | COOL->REST because temperature is below target range and there is natural heating |
| RC2.3 | HEAT->REST because temperature is below target range and there is natural heating |
| RC3.1 | REST->REST because we are in the target range.  There is natural heating so expect temperature to rise |
| RC3.2 | COOL->COOL because we are still within target range and we have natural heating |
| RC3.3 | HEAT->REST because we are in the target range.  There is natural heating so expect temperature to rise |
| RC4.1 | REST->COOL because the temperature is above the target range and we have natural heating |
| RC4.2 | COOL->COOL becuase the temperature is above the target range and we have natural heating |
| RC4.3 | HEAT->COOL because the temperature is above target range and we have natural heating.  (adjust heating lag?) |

#### With natural cooling
| Code | Reason |
| ---- | ------ |
| RC7.1 | REST->HEAT because the temperature is below target range and there is natural cooling |
| RC7.2 | COOL->HEAT because the temperature is below the target range and there is natural cooling (adjust cooling lag?) |
| RC7.3 | HEAT->HEAT because the temperature is below target range and there is natural cooling |
| RC8.1 | REST->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
| RC8.2 | COOL->REST because the temperature is in the target range.  There is natural cooling so expect temperature to fall |
| RC8.3 | HEAT->HEAT because the temperature is still within target range and there is natural cooling |
| RC9.1 | REST->COOL because even though there is natural cooling the temperature is above target range | 
| RC9.2 | COOL->REST because the temperature is above target range and there is natural cooling |
| RC9.3 | HEAT->REST because the temperature is above target range and there is natural cooling |





