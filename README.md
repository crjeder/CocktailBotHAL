# CocktailBotHAL

A Harware Abstraction Layer for remote controll of cocktail mixing robots as a rust trait:

* power on / off robot hardware
* power safe on / off (may be a alias to power)
* status codes:
  * working
  * setup, initial self-test
  * cleaning
  * drink ready
  * idle
* prepared to dispense drink (i. e. glass present)
 * error with error code
* return configuration
 * nr of liquids
* check presence of a glass (optional glass type)
* dispense n liquids in stable measurement "parts"
* return the amout of liquids left (depending on the capability: binary or decimal)
* read configuration (for non volatile storage)
* write configuration (back from storage)

