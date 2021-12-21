#!/bin/bash
#set start = $SECONDS
arduino --verify TempController.ino
#set duration=$(( SECONDS - start ))
#$duration
