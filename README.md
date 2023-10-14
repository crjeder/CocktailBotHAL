# CocktailBotHAL

A Harware Abstraction Layer for cocktail mixing robots
Hardware capabilities:

* power on / off robot hardware
* power safe on / off (may be a alias to power)
* a display (status, mode(e. g. working, setup, cleaning,..), error condition)
* non-volatile storage for configuration
* ability to detect the presence of a glass (optional glass type)
* ability to dispense n liquids in stable measurement "parts"
* (optional) test if liquids are used up / measure remaining volume


## API (with implementaiton hints)

* Bot (low level)
  * initialize
  * power safe on / off (most power consuming hardware is turned off)
  * power on / off (everyting is turned off only "watch dog" stays on)
  * clean (flush the parts wich come into contact with ingredients with water)
  * calibrate (put sensors in a known state)
  * write configuration
  * read configuration (mixer, glasss, recepies, liquids)
  * mix drink recipe (# or reciepe)
  * get status
  * report capabilities ("part" in ml, # of liquids, ..)
 
 private API functions

* Glass
  * take
  * offer
  * test presence
  * get size / volume of glass)
* Mixer
  * get / set number of liquids
  * get / set liquids names
* Dispenser
  * dispense n units -> dispensed volume
  * get liqud level / remaining voulume (summing up dispensed liquid)
* Error conditions
  * wrong glass (identify eg. by weight)
  * no glass (0 weight)
  * contianer empty (valve open, pressure good, pump off and no change in weight)
  * leakage (pump running but no change in pressure)
  * clogging (valve open, pressure good and no change in weight)
  * leak (pump on pressure bad and not raising)
  * power lost (no valid configuration)
  * cleaning required (counting hours / dispensed liquid / clogging)

# Alternative: quick and dirty
* Rest API to dispense liquids
* Static, old fashioned HTML web site to select Cocktails

## Minimal Software Modules
- [X] [HX711 Driver](https://github.com/crjeder/hx711_spi)
- [ ] Scales functionality (tare and calibration)
- [x] [PCA9685 Driver](https://github.com/eldruin/pwm-pca9685-rs)
- [ ] Servo control
- [ ] Pump control (simply drive it when a valve is opened)
- [ ] Simplified UI: Traffic litght Green: drink ready Yellow: ready for new orders, Red: busy
