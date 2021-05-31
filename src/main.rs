#[macro_use] extern crate quick_error;

use std::fmt;
//mod thecocktaildb;
//use thecocktaildb::{Thecocktaildb};
mod cocktail;
use cocktail::{Cocktail, GenericCocktail,convert_measure};
mod cocktailor;
mod thecocktaildb;
mod opendrinks;
use opendrinks::Opendrinks;
use std::io::BufReader;
use std::fs::File;

fn main() -> std::io::Result<()>
{
    println!("converted 13: {}", convert_measure("13").unwrap());
    let f = File::open("/home/christoph/cocktail_recipes/opendrinks/margarita.json")?;
    let mut reader = BufReader::new(f);
    let mut margarita = Opendrinks::default();
    margarita.from_reader(&mut reader)?;

    println!("{:?}", margarita);
    println!("{:?}", margarita.convert_to().unwrap());

    Ok(())
    /*

	let bot = Cocktailbot
	{
		config: Config {display: false},
		dispenser: Dispenser
        {
            max_liquids: 16,
            liquids: vec!
		    [
                Liquid {name: String::from("Vodka"), density: 94, alcohol: 40, suggar: 0},
                Liquid {name: String::from("Gin"), density: 94, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("White Rum"), density: 94, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("Tequila"), density: 94, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("Tripple Sec"), density: 94, alcohol: 40, suggar: 10},
                Liquid {name: String::from("Orange Juice"), density: 104, alcohol: 0, suggar: 8},
                Liquid {name: String::from("Ginger Ale"), density: 104, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("Tonic"), density: 104, alcohol: 40, suggar: 8},
			    Liquid {name: String::from("Lime Juice"), density: 104, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("Coke"), density: 104, alcohol: 0, suggar: 9},
			    Liquid {name: String::from("Prosecco"), density: 94, alcohol: 40, suggar: 0},
			    Liquid {name: String::from("Grenadine"), density: 121, alcohol: 0, suggar: 47},
			    Liquid {name: String::from("Aperol"), density: 100, alcohol: 40, suggar: 47},
			    Liquid {name: String::from("Syrup"), density: 129, alcohol: 0, suggar: 60},
			    Liquid {name: String::from("Amaretto"), density: 100, alcohol: 40, suggar: 47}
		    ]
        },
		glasses: vec!
		[
			Glass {name: String::from("Highball"), volume: 100, weight: 80},
			Glass {name: String::from("Tumbler"), volume: 100, weight: 50}
		],
        cocktails_mixed: 0
	};*/
}

struct Dispenser
{
	pub liquids: Vec<Liquid>,
	max_liquids: u8
}

impl Dispenser
{
	pub fn dispense(nr_liquid: u8, amount: u8) -> Result<u8, BarBotError>
    {
        Ok(amount)
        // BarBotError
        //  OutOfLiquid
        //  Spill
        //  :
    }
}

pub struct Cocktailbot
{
	pub config: Config,
	pub glasses: Vec<Glass>,
	cocktails_mixed: u8,
	dispenser: Dispenser
}

impl Cocktailbot
{
	pub fn mix(&mut self, cocktail: Cocktail) -> Result<u8, BarBotError>
	{
        let mut amout_dispensed = 0u8;

        if self.config.display
        {
            // display(cocktail.name);
        }
		for ingredient in cocktail.ingredients
		{
            amout_dispensed += ingredient.amount;
            // nr = find_dispenser(ingredient);
			// self.dispenser.dispense(nr, );
        }
		self.cocktails_mixed += 1;        // record keeping
        Ok(amout_dispensed)
	}

    // fn display()
}
pub struct Config
{
    display: bool
}

pub struct Glass
{
    name: String,
    volume: u8,                 // in ml
    weight: u8                  // in g
}

#[derive(Debug)]
pub struct Liquid
{
    name: String,
    density: u8,                // in g / cm3 * 100
    alcohol: u8,                // in vol % (ABV)
    suggar: u8,                 // in g / 100 g
}

pub struct BarBotError;

impl fmt::Debug for BarBotError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{{Generic BarBotError in file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Display for BarBotError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "Generic BarBotError")
    }
}
