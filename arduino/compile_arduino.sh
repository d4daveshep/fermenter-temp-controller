#!/bin/bash
#set start = $SECONDS
arduino --verify ./TempController/TempController.ino
#set duration=$(( SECONDS - start ))
#$duration
