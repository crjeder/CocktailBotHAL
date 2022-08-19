# CocktailBotHAL
A Harware Abstraction Layer for cocktail mixing robots

WIP

## API (with implementaiton hints) 
* Bot 
  * initialize
  * disable
  * power off
  * clean
  * calibrate
  * setup liquids
  * write configuration
  * read configuration (mixer, glasss)
  * mix drink recipie (list of liquid # and amount pairs)
  * get status
 
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
  * ~~get liqud level / remaining voulume (summing up dispensed liquid)~~ -> upstream
* Error conditions
  * wrong glass (identify eg. by weight)
  * no glass (0 weight)
  * contianer empty (valve open, pressure good, pump off and no change in weight)
  * leakage (pump running but no change in pressure)
  * clogging (valve open, pressure good and no change in weight)
  * leak (pump on pressure bad and not raising)
  * power lost (no valid configuration)
  * cleaning required (counting hours / dispensed liquid / clogging)
