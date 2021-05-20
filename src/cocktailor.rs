use serde::{Deserialize, Serialize};
use crate::cocktail::{GenericCocktail, convert_measure};
use std::error::Error;

#[derive(Serialize, Deserialize, Default)]
struct Cocktailor
{
    name: String,
    glass: String,
    category: String,
    ingredients: Vec<Ingredient>,
    garnish: String,
    preparation: String
}
#[derive(Serialize, Deserialize, Default)]
struct Ingredient
{
    unit: String,
    amount: u8,
    ingredient: String
}
#[derive(Serialize, Deserialize, Default)]
struct CocktailorDB
{
    cocktails: Vec<Cocktailor>
}

impl GenericCocktail for Cocktailor
// always returns a valid Cocktail struct.
{
    fn convert_to(& self) -> Result<crate::cocktail::Cocktail, Box<dyn Error>>
    {
        let mut converted = crate::cocktail::Cocktail::default();

        converted.name = self.name.clone();
        converted.glass = self.glass.clone();
        converted.category = self.category.clone();
        for i in self.ingredients.iter()
        {
            converted.ingredients.push(crate::cocktail::Ingredient
                {amount: (i.amount * convert_measure(&i.unit)?), name: i.ingredient.clone()});
        };
        converted.garnish = self.garnish.clone();
        converted.shaken_not_stirred = None;
        converted.preparation = self.preparation.clone();

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
}
