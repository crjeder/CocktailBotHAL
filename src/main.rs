#![feature(decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

use std::fmt;
use std::sync::Mutex;

use generic_cocktail::{convert_measure, Cocktail, GenericCocktail};

// use rocket::http::Status;
// use rocket::{data, Data, Outcome, Request, State};
use rocket::{Rocket,State};
use rocket_contrib::json::{Json, JsonValue};
//use rocket::response::content::Json as ContentJason;

use serde::{Deserialize, Serialize};
use serde_json::json;

/// REST API
///
/// base URL:  http://localhost:8000
///
/// get API documentation
/// http://localhost:8000/
#[get("/")]
fn index() -> &'static str { "Hello, world!" }

/// get the liquids the dispenser has loaded
/// http://localhost:8000/REST/get/liquids
#[get("/REST/get/liquids")]
fn get_liquids(l: State<Cocktailbot>) -> Option<Json<&Vec<Liquid>>>
{
    Some(Json(&l.inner().dispenser.liquids))
}

/// get the glasses the bar bot knows abaout
/// http://localhost:8000/REST/get/glasses
#[get("/REST/get/glasses")]
fn get_glasses(l: State<Cocktailbot>) -> Option<Json<&Vec<Glass>>>
{
    Some(Json(&l.inner().glasses))
}

/// have a drink mixed
/// http://localhost:8000/REST/post/cocktail
//#[post("/REST/post/cocktail", data = "<recipe>")]
#[post("/REST/post/cocktail", data = "<r>")]
fn mix_cocktail(
    bot: State<Mutex<Cocktailbot>>,
    r: u8,
) -> Json<&'static str>
{
    let my_bot = &mut bot.inner().lock().unwrap();
    my_bot.cocktails_mixed += r;

    Json
    ("{
        'status': 'success',
        'cocktail': 'test',
        'cocktails_mixed': my_bot.cocktails_mixed
    }")
}

fn main() -> () {
    let this_bot: Mutex<Cocktailbot> = Mutex::new(Cocktailbot {
        config: Config { display: false },
        dispenser: Dispenser {
            max_liquids: 16,
            liquids: vec![
                // density in g / 100 ml, alcohol in vol % = g / 100 ml, suggar g / 100 g (or 100 ml)
                Liquid {
                    name: String::from("Vodka"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Gin"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("White Rum"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Tequila"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Tripple Sec"),
                    density: 94,
                    alcohol: 40,
                    suggar: 10,
                },
                Liquid {
                    name: String::from("Orange Juice"),
                    density: 104,
                    alcohol: 0,
                    suggar: 9,
                },
                Liquid {
                    name: String::from("Ginger Ale"),
                    density: 104,
                    alcohol: 40,
                    suggar: 3,
                },
                Liquid {
                    name: String::from("Tonic"),
                    density: 104,
                    alcohol: 40,
                    suggar: 8,
                },
                Liquid {
                    name: String::from("Lime Juice"),
                    density: 104,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Coke"),
                    density: 104,
                    alcohol: 0,
                    suggar: 9,
                },
                Liquid {
                    name: String::from("Prosecco"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Grenadine"),
                    density: 121,
                    alcohol: 0,
                    suggar: 47,
                },
                Liquid {
                    name: String::from("Aperol"),
                    density: 100,
                    alcohol: 40,
                    suggar: 47,
                },
                Liquid {
                    name: String::from("Syrup"),
                    density: 129,
                    alcohol: 0,
                    suggar: 60,
                },
                Liquid {
                    name: String::from("Amaretto"),
                    density: 100,
                    alcohol: 40,
                    suggar: 47,
                },
            ],
        },
        glasses: vec![
            Glass {
                name: String::from("Highball"),
                volume: 100,
                weight: 80,
            },
            Glass {
                name: String::from("Tumbler"),
                volume: 100,
                weight: 50,
            },
        ],
        cocktails_mixed: 0,
    });
    
    let mut r = rocket::ignite().manage(this_bot);
    r.mount("/", routes![index, get_liquids, get_glasses, mix_cocktail]).launch();
    
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Dispenser
{
    pub liquids: Vec<Liquid>,
    pub(crate) max_liquids: u8,
}

impl Dispenser
{
    pub fn dispense(nr_liquid: u8, amount: u8) -> Result<u8, BarBotError>
    {
        Ok(amount)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cocktailbot
{
    pub config: Config,
    pub glasses: Vec<Glass>,
    pub(crate) cocktails_mixed: u8,
    pub(crate) dispenser: Dispenser,
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
            // nr = find_dispenser(ingredient);#[derive(Serialize, Deserialize, Default, Debug)]
            // self.dispenser.dispense(nr, );
        }
        self.cocktails_mixed += 1; // record keeping
        Ok(amout_dispensed)
    }

    // fn display()
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config
{
    display: bool,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Glass
{
    name: String,
    volume: u8, // in ml
    weight: u8, // in g
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Liquid
{
    name: String,
    density: u8, // in g / cm3 * 100
    alcohol: u8, // in vol % (ABV)
    suggar: u8,  // in g / 100 g
}

#[derive(Serialize, Deserialize)]
pub enum BarBotError
{
    OutOfLiquid(String),
    Spill,
    Generic, //  :
}

impl fmt::Debug for BarBotError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            BarBotError::OutOfLiquid(l) => write!(f, "{{Please refill {}}}", l),
            BarBotError::Spill =>
            {
                write!(f, "Spill detected. Please check glass")
            }
            BarBotError::Generic => write!(
                f,
                "{{Generic BarBotError in file: {}, line: {} }}",
                file!(),
                line!()
            ),
        }
    }
}

impl fmt::Display for BarBotError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self
        {
            BarBotError::OutOfLiquid(l) => write!(f, "{{Please refill {}}}", l),
            BarBotError::Spill =>
            {
                write!(f, "Spill detected. Please check glass")
            }
            BarBotError::Generic => write!(f, "Generic BarBotError"),
        }
    }
}

// Always use a limit to prevent DoS attacks.
const LIMIT: u64 = 1024;
/*
impl data::FromDataSimple for Cocktail
{
    type Error = String;

    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String>
    {
        // Read the data into a String.
        let mut string = String::new();
        if let Err(e) = data.open().take(LIMIT).read_to_string(&mut string)
        {
            return Outcome::Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        let cocktail: Cocktail = match serde_json::from_str(string.to_string().as_ref())
        {
            Err(e) => return Outcome::Failure((Status::UnprocessableEntity, format!("':' {}", e))),
            Ok(c) => c,
        };
        Outcome::Success(cocktail)
    }
}
*/
// curl --data '{"name": "Screwdriver", "glass": "Highball", "category": "All Day", "shaken_not_stirred": true, "ingredients": [{"amount": 60,"name": "Orage Juice"},{"amount": 30,"name": "Vodka"}],"garnish": "Slice of orange","preparation": "Pour everything into a glass over ice."}' http://localhost:8000/REST/post/cocktail
