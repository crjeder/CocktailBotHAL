use serde::{Deserialize, Serialize};
use crate::cocktail::{Cocktail, GenericCocktail, Ingredient, convert_measure};
use reqwest::blocking;
use std::error::Error;

impl GenericCocktail for ThecocktaildbDrink
// always returns a valid Cocktail struct.
{
    fn convert_to(& self) -> Result<Cocktail, Box<dyn Error>>
    {
        let mut converted = Cocktail::default();

        converted.name = self.strDrink.clone();
        converted.glass = self.strGlass.clone();
        if self.strCategory.is_some()
        {
            converted.category = self.strCategory.clone().unwrap();
        };
        converted.shaken_not_stirred = None;  // no field in thecocktaildb
        if self.strIngredient1.is_some() && self.strMeasure1.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(&self.strMeasure1.as_ref().unwrap())?,
              name: self.strIngredient1.clone().unwrap()});
        };
        if self.strIngredient2.is_some() && self.strMeasure2.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure2.as_ref().unwrap())?,
              name: self.strIngredient2.clone().unwrap()});
        };
        if self.strIngredient3.is_some() && self.strMeasure3.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure3.as_ref().unwrap())?,
              name: self.strIngredient3.clone().unwrap()});
        };
        if self.strIngredient4.is_some() && self.strMeasure4.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure4.as_ref().unwrap())?,
              name: self.strIngredient4.clone().unwrap()});
        };
        if self.strIngredient5.is_some() && self.strMeasure5.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure5.as_ref().unwrap())?,
              name: self.strIngredient5.clone().unwrap()});
        };
        if self.strIngredient6.is_some() && self.strMeasure6.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure6.as_ref().unwrap())?,
              name: self.strIngredient6.clone().unwrap()});
        };
        if self.strIngredient7.is_some() && self.strMeasure7.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure7.as_ref().unwrap())?,
              name: self.strIngredient7.clone().unwrap()});
        };
        if self.strIngredient8.is_some() && self.strMeasure8.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure8.as_ref().unwrap())?,
              name: self.strIngredient8.clone().unwrap()});
        };
        if self.strIngredient9.is_some() && self.strMeasure9.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure9.as_ref().unwrap())?,
              name: self.strIngredient9.clone().unwrap()});
        };
        if self.strIngredient10.is_some() && self.strMeasure10.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure10.as_ref().unwrap())?,
              name: self.strIngredient10.clone().unwrap()});
        };
        if self.strIngredient11.is_some() && self.strMeasure11.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure11.as_ref().unwrap())?,
              name: self.strIngredient11.clone().unwrap()});
        };
        if self.strIngredient12.is_some() && self.strMeasure12.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure12.as_ref().unwrap())?,
              name: self.strIngredient12.clone().unwrap()});
        };
        if self.strIngredient13.is_some() && self.strMeasure13.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure13.as_ref().unwrap())?,
              name: self.strIngredient13.clone().unwrap()});
        };
        if self.strIngredient14.is_some() && self.strMeasure14.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure14.as_ref().unwrap())?,
              name: self.strIngredient14.clone().unwrap()});
        };
        if self.strIngredient15.is_some() && self.strMeasure15.is_some()
        {
          converted.ingredients.push(Ingredient {amount: convert_measure(self.strMeasure15.as_ref().unwrap())?,
              name: self.strIngredient15.clone().unwrap()});
        };
        converted.garnish = String::from("");   // no field in thecocktaildb
        converted.preparation = self.strInstructions.clone();

        Ok(converted)  // return
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Thecocktaildb
{
    pub drinks: Vec<ThecocktaildbDrink>
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ThecocktaildbDrink
{
    pub idDrink: String,
    pub strDrink: String,
    pub strDrinkAlternate: Option<String>,
    pub strTags: Option<String>,
    pub strVideo: Option<String>,
    pub strCategory: Option<String>,
    pub strIBA: Option<String>,
    pub strAlcoholic: String,
    pub strGlass: String,
    pub strInstructions: String,
    pub strInstructionsES: Option<String>,
    pub strInstructionsDE: Option<String>,
    pub strInstructionsFR: Option<String>,
    pub strInstructionsIT: Option<String>,
    pub strInstructionsZHHANS: Option<String>,
    pub strInstructionsZHHANT: Option<String>,
    pub strDrinkThumb: Option<String>,
    pub strIngredient1: Option<String>,
    pub strIngredient2: Option<String>,
    pub strIngredient3: Option<String>,
    pub strIngredient4: Option<String>,
    pub strIngredient5: Option<String>,
    pub strIngredient6: Option<String>,
    pub strIngredient7: Option<String>,
    pub strIngredient8: Option<String>,
    pub strIngredient9: Option<String>,
    pub strIngredient10: Option<String>,
    pub strIngredient11: Option<String>,
    pub strIngredient12: Option<String>,
    pub strIngredient13: Option<String>,
    pub strIngredient14: Option<String>,
    pub strIngredient15: Option<String>,
    pub strMeasure1: Option<String>,
    pub strMeasure2: Option<String>,
    pub strMeasure3: Option<String>,
    pub strMeasure4: Option<String>,
    pub strMeasure5: Option<String>,
    pub strMeasure6: Option<String>,
    pub strMeasure7: Option<String>,
    pub strMeasure8: Option<String>,
    pub strMeasure9: Option<String>,
    pub strMeasure10: Option<String>,
    pub strMeasure11: Option<String>,
    pub strMeasure12: Option<String>,
    pub strMeasure13: Option<String>,
    pub strMeasure14: Option<String>,
    pub strImageSource: Option<String>,
    pub strMeasure15: Option<String>,
    pub strImageAttribution: Option<String>,
    pub strCreativeCommonsConfirmed: Option<String>,
    pub dateModified: String
}

impl Thecocktaildb
{
    pub fn from_str(&mut self, s: &str) -> Result<(), serde_json::Error>
    {
        let converted = serde_json::from_str(s)?;
        //*self = serde_json::from_str(s)?       // no error in return value of from_str
        *self = converted;
        Ok(())                                  // return nothing
    }

    pub fn from_api(&mut self, token: &str, id: &str) -> Result<(), reqwest::Error>
    {
        //Runtime::new().expect("Failed to create Tokio runtime");
        let text = blocking::get(format!("https://www.thecocktaildb.com/api/json/v1/{}/lookup.php?i={}", token, id))?
                    .text()?;

        (*self).from_str(&text).unwrap();
        Ok(())
    }
}

/*impl Default for ThecocktaildbDrink
{
    fn default() -> ThecocktaildbDrink
    {
        ThecocktaildbDrink
        {
            idDrink: String::from("0"),
            strDrink: String::from("Empty Glass"),
            strDrinkAlternate: None,
            strTags: None,
            strVideo: None,
            strCategory: None,
            strIBA: None,
            strAlcoholic: String::from("Alcoholic"),
            strGlass: String::from("Empty Glass"),
            strInstructions: String::from("Pour nothing into a glass of your choice."),
            strInstructionsES: None,
            strInstructionsDE: None,
            strInstructionsFR: None,
            strInstructionsIT: None,
            strInstructionsZHHANS: None,
            strInstructionsZHHANT: None,
            strDrinkThumb: None,
            strIngredient1: None,
            strIngredient2: None,
            strIngredient3: None,
            strIngredient4: None,
            strIngredient5: None,
            strIngredient6: None,
            strIngredient7: None,
            strIngredient8: None,
            strIngredient9: None,
            strIngredient10: None,
            strIngredient11: None,
            strIngredient12: None,
            strIngredient13: None,
            strIngredient14: None,
            strIngredient15: None,
            strMeasure1: None,
            strMeasure2: None,
            strMeasure3: None,
            strMeasure4: None,
            strMeasure5: None,
            strMeasure6: None,
            strMeasure7: None,
            strMeasure8: None,
            strMeasure9: None,
            strMeasure10: None,
            strMeasure11: None,
            strMeasure12: None,
            strMeasure13: None,
            strMeasure14: None,
            strImageSource: None,
            strMeasure15: None,
            strImageAttribution: None,
            strCreativeCommonsConfirmed: None,
            dateModified: String::from("-"),
        }
    }
}*/
