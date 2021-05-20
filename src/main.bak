use std::fmt;

fn main()
{
    let screwdriver = Cocktail
    {
        name: String::from("Screwdriver"),
        glass: String::from("Highball"),
        category: String::from("All Day"),
		stir: true,
        ingredients: vec!
        [
          Ingredient {amount: 20, name: String::from("Vodka")},
          Ingredient {amount: 80, name: String::from("Orange Juice")}
        ],
        garnish: String::from("Slice of Orange"),
        preparation: String::from("Pour Vodka and Orange Juice over ice")
    };

	let sunrise = Cocktail
    {
        name: String::from("Tequila Sunrise"),
        glass: String::from("Huricane"),
        category: String::from("Classic"),
		stir: false,
        ingredients: vec!
        [
          Ingredient {amount: 10, name: String::from("Grenadine")},
		  Ingredient {amount: 60, name: String::from("Tequila")},
          Ingredient {amount: 15, name: String::from("Tripple Sec")},
		  Ingredient {amount: 75, name: String::from("Orange Juice")},
		  Ingredient {amount: 20, name: String::from("Lime Juice")}
        ],
        garnish: String::from("Slice of orange and cherry"),
        preparation: String::from("Pour Grenadine over ice and then add the other ingredient slowly. Do not stir!")
    };

    println!("Your Cocktail Choice is:");
    println!("{}", screwdriver.name);
    println!("Ingredients:");
    for liquid in screwdriver.ingredients.iter()
    {
        println!("{} ml {}", liquid.amount, liquid.name);
    }
    println!("{}", screwdriver.preparation);
    println!("Garnish with {}", screwdriver.garnish);
    println!("Enjoy!");

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
	};

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
	pub fn mix(&mut self, cocktail: Cocktail)
	{
		if self.config.display
        {
            // display(cocktail.name);
        }
		for ingredient in cocktail.ingredients
		{
            // nr = find_dispenser(ingredient);
			// self.dispenser.dispense(nr, );
        }
		self.cocktails_mixed += 1;        // record keeping
	}
    // fn display()
}

pub struct Ingredient
{
    amount: u8,                 // in ml
    name: String
}

pub struct Cocktail
{
    name: String,
    glass: String,
    category: String,
    ingredients: Vec<Ingredient>,
    garnish: String,
	stir: bool,
    preparation: String
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
