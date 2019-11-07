# CocktailBotHAL
A Harware Abstraction Layer for cocktail mixing robots

WIP

## API
* Bot 
  * initialize
  * disable
  * clean
  * calibrate
  * setup liquids
  * read configuration
  * mix drink
  * status
* Glass 
  * take
  * offer
  * test presence
  * get size / volume of glass
* Mixer 
  * get number of liquids
  * get liquids names
  * select liquid
* Dispenser 
  * dispense n units
  * get liqud level / remaining voulume
* Error conditions
 * wrong glass
 * no glass
 * contianer empty
 * leakage
 * clogging
 * power lost
 * cleaning required
