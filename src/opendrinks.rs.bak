// module to import recipes from opendrinks

use crate::cocktail::{convert_measure, GenericCocktail};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io;
use std::io::BufRead;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Opendrinks
{
    pub name: String,
    pub description: String,
    pub github: String,
    pub ingredients: Vec<Ingredient>,
    pub directions: Vec<String>,
    pub image: String,
    #[serde(default)] // because source field may be missing
    pub source: String,
    pub keywords: Vec<String>,
}
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Ingredient
{
    pub quantity: String,
    pub measure: String,
    pub ingredient: String,
}

impl GenericCocktail for Opendrinks
// always returns a valid Cocktail struct.
{
    fn convert_to(&self) -> Result<crate::cocktail::Cocktail, Box<dyn Error>>
    {
        let mut converted = crate::cocktail::Cocktail::default();
        converted.name = self.name.clone();
        for i in &self.ingredients
        {
            let mut q: u8 = convert_measure(i.quantity.as_str())?;
            q = q * convert_measure(&i.measure.as_str())?;
            converted.ingredients.push(crate::cocktail::Ingredient {
                amount: q,
                name: i.ingredient.clone(),
            });
        }
        converted.category.clear();
        for k in &self.keywords
        {
            converted.category.push_str(k.as_str());
            converted.category.push_str(", ");
        }
        converted.preparation.clear();
        for d in &self.directions
        {
            converted.preparation.push_str(d.as_str());
            // converted.preparation.push_str(", ");
        }
        Ok(converted)
    }
}

impl Opendrinks
{
    pub fn from_str(&mut self, s: &str) -> Result<(), serde_json::Error>
    {
        let converted = serde_json::from_str(s)?;
        //*self = serde_json::from_str(s)?       // no error in return value of from_str
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
    use crate::cocktail::{Cocktail, GenericCocktail, Ingredient};
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    #[test]
    fn from_file()
    {
        {
            let mut file_w = File::create("testdata/margarita.json").unwrap();
            file_w.write_all
             (
                 r#"
                    {
                        "name": "Margarita",
                        "description": "A margarita is a cocktail consisting of tequila, orange liqueur, and lime juice often served with salt on the rim of the glass.",
                        "github": "alfg",
                        "ingredients":
                        [
                            {
                                "quantity": "1 (6 oz)",
                                "measure": "can",
                                "ingredient": "frozen limeade"
                            },
                            {
                                "quantity": "2",
                                "measure": "fl oz.",
                                "ingredient": "triple sec"
                            },
                            {
                                "quantity": "6",
                                "measure": "fl oz.",
                                "ingredient": "tequila"
                            }
                        ],
                        "directions":
                        [
                            "Fill blender with crushed ice.",
                            "Pour in limeade concentrate, tequila and triple sec.",
                            "Blend until smooth. Pour into glasses and serve."
                        ],
                        "image": "margarita.jpg",
                        "keywords":
                        [
                            "tequila",
                            "alcoholic",
                            "vegan",
                            "simple"
                        ]
                    }
                "#.as_bytes()
            )
            .unwrap();
            file_w.sync_all().unwrap();
        }
        {
            let file_r = File::open("testdata/margarita.json").unwrap();
            let mut reader = BufReader::new(file_r);
            let mut margarita = Opendrinks::default();
            margarita.from_reader(&mut reader).unwrap();
            let my_margarita = Cocktail
             {
                 name: String::from("Margarita"),
                 glass: String::from("Any"),
                 category: String::from("tequila, alcoholic, vegan, simple, "),
                 ingredients: vec!
                 [
                    Ingredient
                    {
                        amount: 180,
                        name: String::from("frozen limeade")
                    },
                    Ingredient
                    {
                        amount: 60,
                        name: String::from("triple sec")
                    },
                    Ingredient
                    {
                        amount: 180,
                        name: String::from("tequila")
                    }
                ],
                garnish: String::from("Slice of thin air"),
                shaken_not_stirred: None,
                preparation: String::from("Fill blender with crushed ice.Pour in limeade concentrate, tequila and triple sec.Blend until smooth. Pour into glasses and serve.")
            };

            assert_eq!(my_margarita, margarita.convert_to().unwrap());
        }
    }
}
