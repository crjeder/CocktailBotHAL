use regex::Regex;
use std::error::Error;
use std::str::FromStr;

pub trait GenericCocktail
{
    fn convert_to(&self) -> Result<Cocktail, Box<dyn Error>>;
}

quick_error! {
    #[derive(Debug)]
    pub enum ConversionError
    {
        IsNegative {}
        IsToBig {}
        UnknownUnit(u: String) {}
    }
}
/*
impl fmt::Display for ConversionError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "Error converting number")
    }
}
*/
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Ingredient
{
    pub amount: u8, // in ml
    pub name: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Cocktail
{
    pub name: String,
    pub glass: String,
    pub category: String,
    pub ingredients: Vec<Ingredient>,
    pub garnish: String,
    pub shaken_not_stirred: Option<bool>,
    pub preparation: String,
}

impl Default for Cocktail
{
    fn default() -> Cocktail
    {
        Cocktail {
            name: String::from("Empty Glass"),
            glass: String::from("Any"),
            category: String::from("All Day"),
            shaken_not_stirred: None,
            ingredients: Vec::new(),
            garnish: String::from("Slice of thin air"),
            preparation: String::from(
                "Pour nothing into a glass of your choice.",
            ),
        }
    }
}

/// A measure consists of a whole number optionally a fraction and a unit. Between whole
/// number and fraction a whitespace is required Between fraction and unit it's optional.
/// at one of whole number and fraction and the unit are required.
/// examples:
///      1 1/2 oz
///        1/3 oz
///      5.5 oz
/// spaces before / after the slash are not allowed
// (?:) denotes a non-capturing group
// (?P<name>) denotes a named capure group
//
pub fn convert_measure(measure: &str) -> Result<u8, Box<dyn Error>>
{
    let res = f64::from_str(measure.trim()); // handle the "number only" case
    if res.is_ok()
    {
        let val = res.unwrap().round();
        if val <= (u8::max_value() as f64)
        {
            return Ok(val as u8);
        }
        else
        {
            return Err(Box::new(ConversionError::IsToBig)); // value > u8:max_value
        }
    };

    let mut value: Option<f64> = None;
    let expr = Regex::new(
        r"^\s*(?:(?P<whole>\d+(?:\.\d+)?)\s+)?(?:(?P<numerator>\d+)/(?P<denominator>\d+))?\s*(?P<unit>[[:alpha:]]+(?:\s+[[:alpha:]]+)?)?",
    )?;

    let captures = expr.captures(measure).unwrap(); // match regex

    if captures.name("whole").is_some()
    // check if capture group "whole" has matched
    {
        // add the whole number part to value
        value = Some(
            captures.name("whole").unwrap().as_str().parse::<f64>().unwrap(),
        );
    };

    // check if there is a fraction (numerator and denominator)
    if captures.name("numerator").is_some()
        && captures.name("denominator").is_some()
    {
        // get numerator's and denominator's value
        let numerator = captures
            .name("numerator")
            .unwrap()
            .as_str()
            .parse::<f64>()
            .unwrap();
        let denominator = captures
            .name("denominator")
            .unwrap()
            .as_str()
            .parse::<f64>()
            .unwrap();

        if denominator > 0.0
        // don't divde by zero
        {
            value = Some(value.unwrap_or(0.0) + numerator / denominator); // add the value of the fractional part
        };
    };

    let mut conversion: f64 = 0.0; // the conversion factor from unit to ml

    if captures.name("unit").is_some()
    // check if there is a match for unit
    {
        let unit = captures.name("unit").unwrap().as_str().to_lowercase();

        conversion = match unit.as_str()
        {
            "oz" | "fl oz" | "ounces" => Ok(30.0),
            "ml" | "milliliters" => Ok(1.0),
            "cl" | "centiliters" => Ok(10.0),
            "dash" | "dashes" => Ok(1.0),
            "tl" | "tsp" | "teaspoon" => Ok(5.0),
            "part" | "parts" | "unit" => Ok(40.0),
            "el" | "tablespoon" | "tablespoons" | "tbsp" => Ok(15.0),
            "shot" | "shots" => Ok(45.0),
            "can" => Ok(6.0 * 30.0),
            "cup" | "cups" => Ok(240.0),
            "pinch" => Ok(0.3),
            "glass" | "glasses" => Ok(250.0),
            "pint" => Ok(473.0),
            "mug" => Ok(355.0),
            "l" | "liter" | "liters" => Ok(1000.0),
            _ => Err(ConversionError::UnknownUnit(String::from(unit))),
        }?;
    }
    else
    // no unit found => assume int's in ml
    {
        conversion = 1.0;
    }

    if value.is_some()
    {
        if value.unwrap() < 0.0
        {
            return Err(Box::new(ConversionError::IsNegative));
        }
        if value.unwrap() * conversion <= (u8::max_value() as f64)
        // overflow
        {
            Ok(((value.unwrap() * conversion).round()) as u8) // converted to ml and return
        }
        else
        {
            return Err(Box::new(ConversionError::IsToBig)); // value > u8:max_value
        }
    }
    else
    {
        return Ok(conversion.round() as u8); // unit without value is trated as
                                             // one unit e. g. "oz" -> 1 oz = 30 ml
                                             // returns 30
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use test_case::test_case;

    #[test_case("3 oz" => 90; "Whole number and unit")]
    #[test_case("1 1/2 oz" => 45; "Reduced fraction greater 1")]
    #[test_case("1/2 oz" => 15; "Reduced fraction")]
    #[test_case("1       oz" => 30; "lots of whitespaces")]
    #[test_case("30/60 oz" => 15; "Reduced fraction with big numbers")]
    #[test_case("oz" => 30; "Unit only")]
    #[test_case("13" => 13; "Number only")]
    #[test_case("7 fl oz" => 210; "Multiword Unit")]
    #[test_case("7/2 oz" => 105; "Improper Fraction")]
    #[test_case("3.5 oz" => 105; "decimal")]
    #[test_case("100 oz" => panics "IsToBig"; "overflow")]
    #[test_case("10.0 X" => panics "UnknownUnit"; "Unknown Unit")]
    // #[test_case("5oz" => 1inconclusive (); "no spaces")]
    fn test_convert(s: &str) -> u8 { convert_measure(s).unwrap() }

    #[test]
    fn from_str()
    {
        let _screwdriver: Cocktail = serde_json::from_str(
            r#"
                {
                    "name": "Screwdriver",
                    "glass": "Highball",
                    "category": "All Day",
                    "shaken_not_stirred": true,
                    "ingredients":
                    [
                        {
                            "amount": 60,
                            "name": "Orage Juice"
                        },
                        {
                            "amount": 30,
                            "name": "Vodka"
                        }
                    ],
                    "garnish": "Slice of orange",
                    "preparation": "Pour everything into a glass over ice."
                }
            "#,
        )
        .unwrap();
    }
}
/*{
"name": "Screwdriver",
"glass": "Highball",
"category": "All Day"{
                    "name": "Screwdriver",
                    "glass": "Highball",
                    "category": "All Day",
                    "shaken_not_stirred": "true",
                    "ingredients":
                    [
                        {
                            "amount": "60",
                            "name": "Orage Juice",
                        },
                        {
                            "amount": "30",
                            "name": "Vodka",
                        },
                    ],
                    "garnish": "Slice of orange",
                    "preparation": "Pour everything into a glass over ice."
                },
"shaken_not_stirred": "true",
"ingredients":
[
{
"amount": "60",
"name": "Orage Juice",
},
{
"amount": "30",
"name": "Vodka",
},
],
"garnish": "Slice of orange",
"preparation": "Pour everything into a glass over ice."
}*/
// {"name": "Screwdriver", "glass": "Highball", "category": "All Day", "shaken_not_stirred": true, "ingredients": [{"amount": 60,"name": "Orage Juice"},{"amount": 30,"name": "Vodka"}],"garnish": "Slice of orange","preparation": "Pour everything into a glass over ice."}
