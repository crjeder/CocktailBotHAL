// module to import recipes from TheCocktailDB

use crate::cocktail::{convert_measure, GenericCocktail, Ingredient, ConversionError};
use reqwest::blocking;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

impl TryFrom<&ThecocktaildbDrink> for GenericCocktail
{
     type Error = ConversionError;

    fn try_from(drink: &ThecocktaildbDrink) -> Result<Self, Self::Error>
    {
        let mut converted = GenericCocktail::default();

        converted.name = drink.strDrink.clone();
        converted.glass = drink.strGlass.clone();
        if drink.strCategory.is_some()
        {
            converted.category = Some(drink.strCategory.clone().unwrap());
        }
        else
        {
            converted.category = None;
        };
        converted.preparation = None; // no field in thecocktaildb
        if drink.strIngredient1.is_some() && drink.strMeasure1.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(&drink.strMeasure1.as_ref().unwrap())?,
                name: drink.strIngredient1.clone().unwrap(),
            });
        };
        if drink.strIngredient2.is_some() && drink.strMeasure2.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure2.as_ref().unwrap())?,
                name: drink.strIngredient2.clone().unwrap(),
            });
        };
        if drink.strIngredient3.is_some() && drink.strMeasure3.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure3.as_ref().unwrap())?,
                name: drink.strIngredient3.clone().unwrap(),
            });
        };
        if drink.strIngredient4.is_some() && drink.strMeasure4.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure4.as_ref().unwrap())?,
                name: drink.strIngredient4.clone().unwrap(),
            });
        };
        if drink.strIngredient5.is_some() && drink.strMeasure5.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure5.as_ref().unwrap())?,
                name: drink.strIngredient5.clone().unwrap(),
            });
        };
        if drink.strIngredient6.is_some() && drink.strMeasure6.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure6.as_ref().unwrap())?,
                name: drink.strIngredient6.clone().unwrap(),
            });
        };
        if drink.strIngredient7.is_some() && drink.strMeasure7.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure7.as_ref().unwrap())?,
                name: drink.strIngredient7.clone().unwrap(),
            });
        };
        if drink.strIngredient8.is_some() && drink.strMeasure8.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure8.as_ref().unwrap())?,
                name: drink.strIngredient8.clone().unwrap(),
            });
        };
        if drink.strIngredient9.is_some() && drink.strMeasure9.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure9.as_ref().unwrap())?,
                name: drink.strIngredient9.clone().unwrap(),
            });
        };
        if drink.strIngredient10.is_some() && drink.strMeasure10.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure10.as_ref().unwrap())?,
                name: drink.strIngredient10.clone().unwrap(),
            });
        };
        if drink.strIngredient11.is_some() && drink.strMeasure11.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure11.as_ref().unwrap())?,
                name: drink.strIngredient11.clone().unwrap(),
            });
        };
        if drink.strIngredient12.is_some() && drink.strMeasure12.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure12.as_ref().unwrap())?,
                name: drink.strIngredient12.clone().unwrap(),
            });
        };
        if drink.strIngredient13.is_some() && drink.strMeasure13.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure13.as_ref().unwrap())?,
                name: drink.strIngredient13.clone().unwrap(),
            });
        };
        if drink.strIngredient14.is_some() && drink.strMeasure14.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure14.as_ref().unwrap())?,
                name: drink.strIngredient14.clone().unwrap(),
            });
        };
        if drink.strIngredient15.is_some() && drink.strMeasure15.is_some()
        {
            converted.ingredients.push(Ingredient {
                amount: convert_measure(drink.strMeasure15.as_ref().unwrap())?,
                name: drink.strIngredient15.clone().unwrap(),
            });
        };
        converted.garnish = None; // no field in thecocktaildb
        converted.preparation_instructions = Some(drink.strInstructions.clone());

        Ok(converted) // return
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Thecocktaildb
{
    pub drinks: Vec<ThecocktaildbDrink>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
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
    pub dateModified: String,
}

impl Thecocktaildb
{
    pub fn from_str(&mut self, s: &str) -> Result<(), serde_json::Error>
    {
        let converted = serde_json::from_str(s)?;
        //*self = serde_json::from_str(s)?       // no error in return value of from_str
        *self = converted;
        Ok(()) // return nothing
    }

    pub fn from_api(&mut self, token: &str, id: &str) -> Result<(), reqwest::Error>
    {
        //Runtime::new().expect("Failed to create Tokio runtime");
        let text = blocking::get(format!(
            "https://www.thecocktaildb.com/api/json/v1/{}/lookup.php?i={}",
            token, id
        ))?
        .text()?;

        (*self).from_str(&text).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::cocktail::GenericCocktail;

    #[test]
    fn str()
    {
        let mut imported = Thecocktaildb::default();
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
                            "strIngredient5":null,"strIngredient6":null,"strIngredient7":null,
                            "strIngredient8":null,"strIngredient9":null,"strIngredient10":null,
                            "strIngredient11":null,"strIngredient12":null,"strIngredient13":null,
                            "strIngredient14":null,"strIngredient15":null,
                            "strMeasure1":"1 1\/2 oz ",
                            "strMeasure2":"1\/2 oz ",
                            "strMeasure3":"1 oz ",
                            "strMeasure4":null,
                            "strMeasure5":null,"strMeasure6":null,"strMeasure7":null,
                            "strMeasure8":null,"strMeasure9":null,"strMeasure10":null,
                            "strMeasure11":null,"strMeasure12":null,"strMeasure13":null,
                            "strMeasure14":null,"strMeasure15":null,
                            "strImageSource":"https:\/\/commons.wikimedia.org\/wiki\/File:Klassiche_Margarita.jpg",
                            "strImageAttribution":"Cocktailmarler",
                            "strCreativeCommonsConfirmed":"Yes",
                            "dateModified":"2015-08-18 14:42:59"
                        }
                    ]
                }
            "#
        ).unwrap();

        let magerita: GenericCocktail = GenericCocktail
        {
            name: String::from("Margarita"),
            glass: String::from("Cocktail glass"),
            category: Some(String::from("Ordinary Drink")),
            preparation: None,
            ingredients: vec!
            [
                Ingredient {amount: 45, name: String::from("Tequila")},
                Ingredient {amount: 15, name: String::from("Triple sec")},
                Ingredient {amount: 30, name: String::from("Lime juice")},
                // Ingredient {amount: 80, name: String::from("Salt")} is not converted since no measure is given
            ],
            garnish: None,
            preparation_instructions: Some(String::from(
                "Rub the rim of the glass with the lime slice to make the salt stick to it. \
                Take care to moisten only the outer rim and sprinkle the salt on it. \
                The salt should present to the lips of the imbiber and never mix into the cocktail. \
                Shake the other ingredients with ice, then carefully pour into the glass."))
        };

        assert_eq!(magerita, GenericCocktail::try_from(&imported.drinks[0]).unwrap());
    }

    #[test]
    #[ignore]
    fn api()
    {
        let mut web: Thecocktaildb = Default::default();
        web.from_api("1", "11007").unwrap();

        let magerita: GenericCocktail = GenericCocktail
        {
            name: String::from("Margarita"),
            glass: String::from("Cocktail glass"),
            category: Some(String::from("Ordinary Drink")),
            preparation: None,
            ingredients: vec!
            [
                Ingredient {amount: 45, name: String::from("Tequila")},
                Ingredient {amount: 15, name: String::from("Triple sec")},
                Ingredient {amount: 30, name: String::from("Lime juice")},
                // Ingredient {amount: 80, name: String::from("Salt")} is not converted since no measure is given
            ],
            garnish: None,
            preparation_instructions: Some(String::from(
                "Rub the rim of the glass with the lime slice to make the salt stick to it.
                Take care to moisten only the outer rim and sprinkle the salt on it.
                The salt should present to the lips of the imbiber and never mix into the cocktail.
                Shake the other ingredients with ice, then carefully pour into the glass."))
        };

        assert_eq!(magerita, GenericCocktail::try_from(&web.drinks[0]).unwrap());
    }
}
