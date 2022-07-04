/* for rocket
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
*/
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate quick_error;

use std::fmt;
//use std::sync::Mutex;

mod cocktail;
mod import;
use crate::cocktail::GenericCocktail;
use crate::import::leosbar::*;

use std::fs::File;
//use std::io::prelude::*;
use std::io::BufReader;
pub use select::{document::Document, predicate::Name};

use spider::website::Website;

fn from_html()
{

        let file_r = File::open("testdata/Willkommen in LEOs Bar.html").unwrap();
        let reader = BufReader::new(file_r);
        let doc = Document::from_read(reader).unwrap();

        let mut margarita = Leosbar::default();
        margarita.from_document(&doc).unwrap();
        GenericCocktail::try_from(&margarita).unwrap();

}

fn main() -> std::io::Result<()>
{
    from_html();
    // let mut website: Website = Website::new("http://bar.leo.org/dx");
    let mut website: Website = Website::new("https://choosealicense.com");
    //website.configuration.blacklist_url.push("http://bar.leo.org/".to_string());
    website.configuration.delay = 2000;
    website.configuration.verbose = true;
    website.crawl();

    for page in website.get_pages()
    {
        println!("* {}", page.get_url());
        //let mut drink = Website::new(page.get_url().as_str());
        //drink.crawl();
        //let document = Document::from(page.get_url().as_str());
        //let drink = GenericCocktail::try_from(&document).unwrap();
        //println!("{:?}", drink);
    }


    Ok(())
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
    pub fn mix(&mut self, cocktail: GenericCocktail) -> Result<u8, BarBotError>
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
