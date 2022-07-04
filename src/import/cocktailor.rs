// module to import recipes from Cocktailor

use crate::cocktail::{convert_measure, GenericCocktail, ConversionError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Default)]
struct Cocktailor
{
    name: String,
    glass: String,
    category: String,
    ingredients: Vec<Ingredient>,
    garnish: String,
    preparation: String,
}
#[derive(Serialize, Deserialize, Default)]
struct Ingredient
{
    unit: String,
    amount: f32,
    ingredient: String,
}

//type CocktailorDB = Vec<Cocktailor>;
#[derive(Serialize, Deserialize, Default)]
struct CocktailorDB
{
    cocktail: Vec<Cocktailor>,
}

impl TryFrom<&Cocktailor> for GenericCocktail
{
    type Error = ConversionError;

    fn try_from(drink: &Cocktailor) -> Result<Self, Self::Error>
    {
        let mut converted = crate::cocktail::GenericCocktail::default();

        converted.name = drink.name.clone();
        converted.glass = drink.glass.clone();
        converted.category = Some(drink.category.clone());
        for i in drink.ingredients.iter()
        {
            converted.ingredients.push(crate::cocktail::Ingredient {
                amount: (i.amount * f32::from(convert_measure(&i.unit)?)).round() as u8,
                name: i.ingredient.clone(),
            });
        }
        converted.garnish = Some(drink.garnish.clone());
        converted.preparation = None;
        converted.preparation_instructions = Some(drink.preparation.clone());

        Ok(converted)
    }
}

impl CocktailorDB
{
    pub fn from_str(&mut self, s: &str) -> Result<(), Box<dyn Error>>
    {
        *self = serde_json::from_str(s)?;
        Ok(())
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
            let mut file_w = File::create("testdata/vesper.json").unwrap();
            file_w.write_all(
                r#"
                {
                    "cocktail":
                    [
                        {
                            "name": "Vesper",
                            "glass": "martini",
                            "category": "Before Dinner Cocktail",
                            "ingredients":
                            [
                                {
                                    "unit": "cl",
                                    "amount": 6,
                                    "ingredient": "Gin"
                                },
                                {
                                     "unit": "cl",
                                     "amount": 1.5,
                                     "ingredient": "Vodka"
                                 },
                                 {
                                     "unit": "cl",
                                     "amount": 0.75,
                                     "ingredient": "Lillet Blonde"
                                 }
                            ],
                            "garnish": "Lemon twist",
                            "preparation": "Shake and strain into a chilled cocktail glass."
                        }
                    ]
                }
                "#
                .as_bytes()
            )
            .unwrap();

            file_w.sync_all().unwrap();
        }
        {
            let file_r = File::open("testdata/vesper.json").unwrap();
            let mut reader = BufReader::new(file_r);
            let mut vesper = CocktailorDB::default();
            vesper.from_reader(&mut reader).unwrap();
            let my_vesper = GenericCocktail
             {
                 name: String::from("Vesper"),
                 glass: String::from("martini"),
                 category: Some(String::from("Before Dinner Cocktail")),
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

            assert_eq!(my_vesper, GenericCocktail::try_from(&vesper.cocktail[0]).unwrap());
        }
    }
}
