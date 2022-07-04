// module to import / export recipes from / to IBA Cocktail Browser

use crate::cocktail::{convert_measure, GenericCocktail, ConversionError};
use serde::{Deserialize, Serialize};
use std::io;
use std::io::BufRead;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CocktailBrowserIngredient
{
    pub unit: String,
    pub amount: f32,
    pub ingredient: String
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CocktailBrowser
{
    pub is_iba: bool,
    pub name: String,
    pub colors: String,
    pub glass: String,
    pub ingredients: Vec<CocktailBrowserIngredient>,
    pub garnish: String,
    pub preparation: String
}

impl TryFrom<&CocktailBrowser> for GenericCocktail
{
    type Error = ConversionError;

    fn try_from(drink: &CocktailBrowser) -> Result<Self, Self::Error>
    {
        let mut converted = crate::cocktail::GenericCocktail::default();
        converted.name = drink.name.clone();
        for i in &drink.ingredients
        {
            let unit = convert_measure(i.unit.as_str())? as f32;
            converted.ingredients.push(crate::cocktail::Ingredient
            {
                amount: cast::u8((i.amount * unit).round())?,
                name: i.ingredient.clone(),
            });
        }
        converted.category = None;
        converted.preparation_instructions = Some(drink.preparation.clone());
        converted.garnish = Some(drink.garnish.clone());
        converted.glass = drink.glass.clone();

        Ok(converted)
    }
}

impl CocktailBrowser
{
    pub fn from_str(&mut self, s: &str) -> Result<(), serde_json::Error>
    {
        let converted = serde_json::from_str(s)?;
        *self = converted;
        Ok(()) // return nothing
    }

    pub fn from_reader<R: BufRead>(&mut self, reader: &mut R)
        -> io::Result<()>
    {
        *self = serde_json::from_reader(reader)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cocktail::{GenericCocktail, Ingredient};
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    #[test]
    fn from_file()
    {
        {
            let mut file_w = File::create("testdata/vesper2.json").unwrap();
            file_w.write_all
             (
                 r#"
                    {
                        "is_iba": true,
                        "name": "Vesper",
                        "colors": " #D88317",
                        "glass": "martini",
                        "category": "Before Dinner Cocktail",
                        "ingredients":
                        [
                            { "unit": "cl", "amount": 6, "ingredient": "Gin" },
                            { "unit": "cl", "amount": 1.5, "ingredient": "Vodka" },
                            { "unit": "cl", "amount": 0.75, "ingredient": "Lillet Blonde" }
                        ],
                        "garnish": "Lemon twist",
                        "preparation": "Shake and strain into a chilled cocktail glass."
                    }
                "#.as_bytes()
            )
            .unwrap();
            file_w.sync_all().unwrap();
        }
        {
            let file_r = File::open("testdata/vesper2.json").unwrap();
            let mut reader = BufReader::new(file_r);
            let mut vesper = CocktailBrowser::default();
            vesper.from_reader(&mut reader).unwrap();
            let my_vesper = GenericCocktail
             {
                 name: String::from("Vesper"),
                 glass: String::from("martini"),
                 category: None,
                 ingredients: vec!
                 [
                    Ingredient
                    {
                        amount: 60,
                        name: String::from("Gin")
                    },
                    Ingredient
                    {
                        amount: 15,
                        name: String::from("Vodka")
                    },
                    Ingredient
                    {
                        amount: 8,
                        name: String::from("Lillet Blonde")
                    }
                ],
                garnish: Some(String::from("Lemon twist")),
                preparation: None,
                preparation_instructions: Some(String::from("Shake and strain into a chilled cocktail glass."))
            };

            assert_eq!(my_vesper, GenericCocktail::try_from(&vesper).unwrap());
        }
    }
}
