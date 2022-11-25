// Example:
// https://www.socialandcocktail.co.uk/cocktails/agent-orange/

pub use select::{document::Document, predicate::Name};
pub use std::convert::TryFrom;
use crate::cocktail::{convert_measure, GenericCocktail, Ingredient, ConversionError};
//use  std::num::ParseFloatError;

quick_error! {
    #[derive(Debug)]
    /// Errors which can happend during measure conversion
    pub enum SocialError
    {
        /// the HTML does not represent a valid recipe
        NotARecipe(s: String) {}
        /// Critical error (i. e. Regex failure)
        InternalError {}
        /// Error parsing a number form string i. e. ParseFloatError
        ParseError(s: String) {}
        /// Error in casting numbers
        CastError(err: cast::Error) {}
        /// Error converting measures
        ConversionError(err: crate::cocktail::ConversionError)
    }
}

impl From<std::num::ParseFloatError> for SocialError
{
    fn from(_error: std::num::ParseFloatError) -> Self
    {
        SocialError::ParseError("Error parsing float".to_string())
    }
}

impl From<cast::Error> for SocialError
{
    fn from(error: cast::Error) -> Self
    {
        SocialError::CastError(error)
    }
}

impl From<crate::cocktail::ConversionError> for SocialError
{
    fn from(error: crate::cocktail::ConversionError) -> Self
    {
        SocialError::ConversionError(error)
    }
}

#[derive(Default, Debug, Clone)]
pub struct SocialCocktails
{
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub preparation_instructions: Option<String>,
    pub glass: String
}

impl SocialCocktails
{
    pub fn from_document(&mut self, document: &Document)-> Result<Self, SocialError>
    {
        let mut ingredients = Vec::new();
        // Xpath
        //*[@id="content-home"]/section
        // /html/body/div[3]/div[2]/div/div[3]/div/section
        // Selector:
        // #content-home > section
        //
        // Title
        // find(Class("single_header")
        //
        // Recipe
        // find(Class("recipe-content")
        // Glass
        // find(Class("cocktail-terms")
      
        // >>>>> ab hier noch nicht geändert
        
        let mut recipe = document.find(Class("recipe-content"))?;
        let name = recipe.next().ok_or(LeosError::NotARecipe("No receipe name found".to_string()))?.text();

        let table = document.find(Name("table")).nth(2).ok_or(LeosError::NotARecipe("table of ingredients not found".to_string()))?;

        for row in table.find(Name("tr"))
        {
            let mut cel = row.find(Name("td"));
            let mut c = row.clone().find(Name("td"));
            println!("cel {:?}", c.next());
            // let amount = format!("{} {}",cel.nth(1).expect("1").text(), cel.next().expect("2").text());
            //println!(">{}< >{}< >{}<", cel.next().expect("1").text(), cel.nth(1).expect("2").text(), cel.next().expect("1").text());
            let v = cel.next()
                .ok_or(LeosError::NotARecipe("no cell containging the amount of liquid found".to_string()))?
                .text()
                .parse::<f64>(); // type Result: error means the ingredient has no amount and
                                 // will be treated as garnish
            let unit_conversion = convert_measure
            (
                cel.nth(1)
                .ok_or(LeosError::NotARecipe("no cell containging the unit of measure found".to_string()))?
                .text()
                .as_str()
            );              // type Result: if ingredient has unknown unit it
                             // will be treated as garnish

            //println!("factor = {:?}", unit_conversion);

            match unit_conversion
            {
                Ok(factor) =>
                {
                    let value = v.unwrap_or(0.0) as u8; // should be garnsh on error
                    ingredients.push
                    (
                        Ingredient
                        {
                            name: cel.next().ok_or(LeosError::NotARecipe("no cell containging the name of liquid found".to_string()))?.text(),
                            amount: value * factor
                        }
                    );
                },
                Err(ConversionError::UnknownUnit(unit)) =>         // garnish
                {
                    if v.is_ok()
                    {
                        let value = v.unwrap() as u8;           // should not panic because v is ok
                        let g = format!("{} {}, {}", value, unit,
                            cel.next()
                            .ok_or(LeosError::ParseError(unit.clone()))?
                            .text()
                        );
                        garnish = Some(String::from(g));
                    }
                    else
                    {
                        let  g = format!("{} {}, ", unit,
                            cel.next()
                            .ok_or(LeosError::ParseError(unit.clone()))?
                            .text()
                        );
                        garnish = Some(g);
                    }
                }
                Err(ConversionError::ParseError) =>         // e. g. no unit found
                {
                    garnish = Some(cel.next()
                                    .ok_or(LeosError::ParseError("There should be some garnish".to_string()))? // make garnsh optional
                                    .text());
                }
                Err(e) => return Err(LeosError::ConversionError(e))
            }
        }


        let paragraph = document.find(Name("strong")).next().ok_or(LeosError::NotARecipe("Preparation instructions expected".to_string()))?;
        let zubereitung = String::from("Zubereitung");

        println!("paragraph = {:?}", paragraph.next());

        if paragraph.text() == zubereitung
        {
            self.preparation_instructions = Some(paragraph.next().ok_or(LeosError::NotARecipe("Preparation instructions expected".to_string()))?.text());
            // make preparation instructions optional (no error but None())
        }
        else
        {
            return Err(LeosError::NotARecipe("Preparation instructions expected".to_string()));
        };

        self.name = name;
        self.ingredients = ingredients;
        self.garnish = garnish;
        //self.preparation_instructions = instructions;

        Ok(self.clone())
    }
}

impl TryFrom<&Leosbar> for GenericCocktail
{
    type Error = ConversionError;

    fn try_from(drink: &Leosbar) -> Result<Self, Self::Error>
    {

        Ok
        (
            GenericCocktail
            {
                name: drink.name.clone(),
                glass: String::from("Cocktail"),
                category: None,
                ingredients: drink.ingredients.clone(),
                garnish: drink.garnish.clone(),
                preparation: None,
                preparation_instructions: drink.preparation_instructions.clone()
            }
        )

    }
}


#[cfg(test)]
mod tests
{
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use crate::cocktail::GenericCocktail;

    #[test]
    fn from_html()
    {
        {
            let file_r = File::open("testdata/Willkommen in LEOs Bar.html").unwrap();
            let reader = BufReader::new(file_r);
            let doc = Document::from_read(reader).unwrap();

            let mut margarita = Leosbar::default();
            margarita.from_document(&doc).unwrap();
            let my_margarita = GenericCocktail
             {
                 name: String::from("Margarita"),
                 glass: String::from("Cocktail"),
                 category: None,
                 ingredients: vec!
                 [
                    Ingredient
                    {
                        amount: 40,
                        name: String::from("Tequila")
                    },
                    Ingredient
                    {
                        amount: 20,
                        name: String::from("Cointreau")
                    },
                    Ingredient
                    {
                        amount: 20,
                        name: String::from("Zitronensaft")
                    }
                ],
                garnish: Some(String::from("Zitronenviertel")),
                preparation: None,
                preparation_instructions: Some(String::from("Zitronenviertel am Rand des Glases entlang
                fahren, auf eine mit Salz gefüllte Untertasse
                stülpen. Überflüssiges Salz abschütteln.
                Zutaten auf viel Eis mixen."))
            };

            assert_eq!(my_margarita, GenericCocktail::try_from(&margarita).unwrap());
        }
    }
}
