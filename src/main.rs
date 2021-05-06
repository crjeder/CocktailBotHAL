use std::fmt;
use std::str::FromStr;
use regex::Regex;
use serde::{Deserialize, Serialize};

fn main()
{
    /* TEST
    let mut measure: String;
    measure = String::from("1 1/2 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("1/2 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("10  oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("1 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("10 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("1    oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("90/60 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    measure = String::from("1.5 oz"); println!("{} = {} ml", measure, convert_measure(&measure.as_str()));
    */

    let imported: Thecocktaildb = serde_json::from_str
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

    let mut converted = Cocktail::default();
    converted.name = imported.drinks[0].strDrink.clone();
    converted.glass = imported.drinks[0].strGlass.clone();
    converted.category = imported.drinks[0].strCategory.clone().unwrap();
    converted.stir = true;  // no field in thecocktaildb
    if imported.drinks[0].strIngredient1.is_some() && imported.drinks[0].strMeasure1.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure1.clone().unwrap()),
          name: imported.drinks[0].strIngredient1.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient2.is_some() && imported.drinks[0].strMeasure2.is_some()
    {
        println!("{}", imported.drinks[0].strMeasure2.as_ref().unwrap());
      converted.ingredients.push(Ingredient {amount: convert_measure(imported.drinks[0].strMeasure2.as_ref().unwrap()),
          name: imported.drinks[0].strIngredient2.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient3.is_some() && imported.drinks[0].strMeasure3.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure3.clone().unwrap()),
          name: imported.drinks[0].strIngredient3.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient4.is_some() && imported.drinks[0].strMeasure4.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure4.clone().unwrap()),
          name: imported.drinks[0].strIngredient4.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient5.is_some() && imported.drinks[0].strMeasure5.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure5.clone().unwrap()),
          name: imported.drinks[0].strIngredient5.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient6.is_some() && imported.drinks[0].strMeasure6.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure6.clone().unwrap()),
          name: imported.drinks[0].strIngredient6.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient7.is_some() && imported.drinks[0].strMeasure7.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure7.clone().unwrap()),
          name: imported.drinks[0].strIngredient7.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient8.is_some() && imported.drinks[0].strMeasure8.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure8.clone().unwrap()),
          name: imported.drinks[0].strIngredient8.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient9.is_some() && imported.drinks[0].strMeasure9.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure9.clone().unwrap()),
          name: imported.drinks[0].strIngredient9.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient10.is_some() && imported.drinks[0].strMeasure10.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure10.clone().unwrap()),
          name: imported.drinks[0].strIngredient10.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient11.is_some() && imported.drinks[0].strMeasure11.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure11.clone().unwrap()),
          name: imported.drinks[0].strIngredient11.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient12.is_some() && imported.drinks[0].strMeasure12.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure12.clone().unwrap()),
          name: imported.drinks[0].strIngredient12.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient13.is_some() && imported.drinks[0].strMeasure13.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure13.clone().unwrap()),
          name: imported.drinks[0].strIngredient13.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient14.is_some() && imported.drinks[0].strMeasure14.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure14.clone().unwrap()),
          name: imported.drinks[0].strIngredient14.clone().unwrap()});
    };
    if imported.drinks[0].strIngredient15.is_some() && imported.drinks[0].strMeasure15.is_some()
    {
      converted.ingredients.push(Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure15.clone().unwrap()),
          name: imported.drinks[0].strIngredient15.clone().unwrap()});
    };
    converted.garnish = String::from("");
    converted.preparation = imported.drinks[0].strInstructions.clone();

    println!("{:?}", converted);

    /*
    let converted = Cocktail
    {
        name: imported.drinks[0].strDrink.clone(),
        glass: imported.drinks[0].strGlass.clone(),
        category: imported.drinks[0].strCategory.clone().unwrap(),
        stir: true,  // no field in thecocktaildb
        ingredients: vec!
        [
          Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure1), name: imported.drinks[0].strIngredient1.clone()},
          Ingredient {amount: convert_measure(&imported.drinks[0].strMeasure2), name: imported.drinks[0].strIngredient2.clone()}
        ],
        garnish: String::from(""),
        preparation: imported.drinks[0].strInstructions.clone()
    };*/

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

#[derive(Debug)]
pub struct Ingredient
{
    amount: u8,                 // in ml
    name: String
}

#[derive(Debug)]
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

impl Default for Cocktail
{
    fn default() -> Cocktail
    {
        Cocktail
        {
            name: String::from("Empty Glass"),
            glass: String::from("Any"),
            category: String::from("All Day"),
    		stir: false,
            ingredients: Vec::new(),
            garnish: String::from("Slice of thin air"),
            preparation: String::from("Pour nothing into a glass of your choice.")
        }
    }
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

fn convert_measure(measure: &str) -> u8
{
    // a measure consists of a whole number optionally a fraction and a unit. Between whole
    // number and fraction a whitespace is required Between fraction and unit it's optional.
    // at one of whole number and fraction and the unit are required.
    // examples:
    //      1 1/2 oz
    //        1/3 oz
    //      5 oz
    // spaces before / after the slash are not allowed
    // (?:) denotes a non-capturing group
    // (?P<name>) denotes a named capure group
    //
    // the regex is:
    //  ^\s*(?:(?P<whole>\d+){0,1}\s+){0,1}(?:(?P<numerator>\d+)/(?P<denominator>\d+)){0,1}\s*(?P<unit>\w+)
    //
    let expr = Regex::new(r"^\s*(?:(?P<whole>\d+){0,1}\s+){0,1}(?:(?P<numerator>\d+)/(?P<denominator>\d+)){0,1}\s*(?P<unit>\w+)").unwrap();
    let captures = expr.captures(measure).unwrap();     // match regex

    let mut value: f64 = 0.0;

    if captures.name("whole").is_some()                 // check if capture group "whole" has matched
    {
        // add the whole number part to value
        value += captures.name("whole").unwrap().as_str().parse::<f64>().unwrap();
    };

    // check if there is a fraction (numerator and denominator)
    if captures.name("numerator").is_some() && captures.name("denominator").is_some()
    {
        // get numerator's and denominator's value
        let numerator = captures.name("numerator").unwrap().as_str().parse::<f64>().unwrap();
        let denominator = captures.name("denominator").unwrap().as_str().parse::<f64>().unwrap();

        if denominator > 0.0                    // don't divde by zero
        {
            value += numerator / denominator;   // add the value of the fractional part
        };
    };

    let mut conversion: f64 = 0.0;              // the conversion factor from unit to ml

    if captures.name("unit").is_some()          // check if there is a match for unit
    {
        conversion = match captures.name("unit").unwrap().as_str().to_lowercase().as_str()
        {
            "oz" => 30.0,
            "ml" => 1.0,
            "cl" => 10.0,
            "dash" => 1.0,
            "tl" => 5.0,
            "bl" => 2.0,
            "part" => 40.0,
            "el" => 15.0,
            "shot" => 45.0,
            _ => 0.0,
        };

            /*
                1 oz = 30 ml,
                1 ml = 1 ml,
                1 cl = 10 ml,
                1 l = 1000 ml,
                1 Dash = 1 ml,
                1 Tl = 5 ml,
                1 Bl = 2 ml,
                1 Part = 40 ml,
                1 Jigger = 42 ml,
                1 El = 15 ml,
                1 Gill = 36 ml,
                1 Pony = 30 ml,
                1 Schuss = 1 ml,
                1 Short = 20 ml,
                1 Shot = 45 ml

            */
    };

    if value > 0.0 && value <= (u8::max_value() as f64) // is the value valid?
    {
        ((value * conversion).round()) as u8            // converted to ml and return
    }
    else
    {
        0                                               // something went wrong
    }
}

#[derive(Serialize, Deserialize)]
struct Thecocktaildb
{
    drinks: Vec<ThecocktaildbDrink>
}

#[derive(Serialize, Deserialize)]
struct ThecocktaildbDrink
{
    idDrink: String,
    strDrink: String,
    strDrinkAlternate: Option<String>,
    strTags: String,
    strVideo: Option<String>,
    strCategory: Option<String>,
    strIBA: Option<String>,
    strAlcoholic: String,
    strGlass: String,
    strInstructions: String,
    strInstructionsES: Option<String>,
    strInstructionsDE: Option<String>,
    strInstructionsFR: Option<String>,
    strInstructionsIT: Option<String>,
    strInstructionsZHHANS: Option<String>,
    strInstructionsZHHANT: Option<String>,
    strDrinkThumb: Option<String>,
    strIngredient1: Option<String>,
    strIngredient2: Option<String>,
    strIngredient3: Option<String>,
    strIngredient4: Option<String>,
    strIngredient5: Option<String>,
    strIngredient6: Option<String>,
    strIngredient7: Option<String>,
    strIngredient8: Option<String>,
    strIngredient9: Option<String>,
    strIngredient10: Option<String>,
    strIngredient11: Option<String>,
    strIngredient12: Option<String>,
    strIngredient13: Option<String>,
    strIngredient14: Option<String>,
    strIngredient15: Option<String>,
    strMeasure1: Option<String>,
    strMeasure2: Option<String>,
    strMeasure3: Option<String>,
    strMeasure4: Option<String>,
    strMeasure5: Option<String>,
    strMeasure6: Option<String>,
    strMeasure7: Option<String>,
    strMeasure8: Option<String>,
    strMeasure9: Option<String>,
    strMeasure10: Option<String>,
    strMeasure11: Option<String>,
    strMeasure12: Option<String>,
    strMeasure13: Option<String>,
    strMeasure14: Option<String>,
    strMeasure15: Option<String>,
    strImageSource: Option<String>,
    strImageAttribution: Option<String>,
    strCreativeCommonsConfirmed: Option<String>,
    dateModified: String
}
