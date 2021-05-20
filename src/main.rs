#[macro_use] extern crate quick_error;

use std::fmt;
mod thecocktaildb;
use thecocktaildb::{Thecocktaildb};
mod cocktail;
use cocktail::{Cocktail, Ingredient, GenericCocktail};
mod cocktailor;

fn main()
{
    let mut imported: Thecocktaildb = Default::default();// serde_json::from_str
    imported.from_str
    (
        r#"
            {
                "drinks":
                [
                    {
                        "idDrink":"11007",
                        "strDrink":"Margarita",
                        "strDrinkAlternate":null,
                        "strTags":"IBA, ContemporaryClassic",
                        "strVideo":"null",
                        "strCategory":"Ordinary Drink",
                        "strIBA":"Contemporary Classics",
                        "strAlcoholic":"Alcoholic",
                        "strGlass":"Cocktail glass",
                        "strInstructions":"Rub the rim of the glass with the lime slice to make the salt stick to it. Take care to moisten only the outer rim and sprinkle the salt on it. The salt should present to the lips of the imbiber and never mix into the cocktail. Shake the other ingredients with ice, then carefully pour into the glass.",
                        "strInstructionsES":"null",
                        "strInstructionsDE":"Reiben Sie den Rand des Glases mit der Limettenscheibe, damit das Salz daran haftet. Achten Sie darauf, dass nur der \u00e4u\u00dfere Rand angefeuchtet wird und streuen Sie das Salz darauf. Das Salz sollte sich auf den Lippen des Genie\u00dfers befinden und niemals in den Cocktail einmischen. Die anderen Zutaten mit Eis sch\u00fctteln und vorsichtig in das Glas geben.",
                        "strInstructionsFR":null,
                        "strInstructionsIT":"Strofina il bordo del bicchiere con la fetta di lime per far aderire il sale.\r\nAvere cura di inumidire solo il bordo esterno e cospargere di sale.\r\nIl sale dovrebbe presentarsi alle labbra del bevitore e non mescolarsi mai al cocktail.\r\nShakerare gli altri ingredienti con ghiaccio, quindi versarli delicatamente nel bicchiere.",
                        "strInstructionsZH-HANS":null,
                        "strInstructionsZH-HANT":null,
                        "strDrinkThumb":"https:\/\/www.thecocktaildb.com\/images\/media\/drink\/5noda61589575158.jpg",
                        "strIngredient1":"Tequila",
                        "strIngredient2":"Triple sec",
                        "strIngredient3":"Lime juice",
                        "strIngredient4":"Salt",
                        "strIngredient5":null,
                        "strIngredient6":null,"strIngredient7":null,"strIngredient8":null,"strIngredient9":null,"strIngredient10":null,"strIngredient11":null,"strIngredient12":null,"strIngredient13":null,"strIngredient14":null,"strIngredient15":null,"strMeasure1":"1 1\/2 oz ","strMeasure2":"1\/2 oz ","strMeasure3":"1 oz ","strMeasure4":null,"strMeasure5":null,"strMeasure6":null,"strMeasure7":null,"strMeasure8":null,"strMeasure9":null,"strMeasure10":null,"strMeasure11":null,"strMeasure12":null,"strMeasure13":null,"strMeasure14":null,"strMeasure15":null,"strImageSource":"https:\/\/commons.wikimedia.org\/wiki\/File:Klassiche_Margarita.jpg","strImageAttribution":"Cocktailmarler","strCreativeCommonsConfirmed":"Yes","dateModified":"2015-08-18 14:42:59"}]}
        "#
    ).unwrap();

    // println!("{:?}", imported.drinks[0].convert_to());

    let mut web: Thecocktaildb = Default::default();
    web.from_api("1","11007").unwrap();
    println!("{:?}", web.drinks[0].convert_to());
    // assert_eq!(imported, web);  // implement ParialEq first
/*
    let screwdriver = Cocktail
    {
        name: String::from("Screwdriver"),
        glass: String::from("Highball"),
        category: String::from("All Day"),
		shaken_not_stirred: Some(true),
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
*/
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
