use regex::Regex;
use std::error::Error;
use std::num::ParseIntError;
use std::fmt;



pub trait GenericCocktail
{
    fn convert_to(& self) -> Result<Cocktail, Box<dyn Error>>;
}

quick_error!
{
    #[derive(Debug, Clone)]
    pub enum ConversionError
    {
        ValueToBig {}
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
#[derive(Debug, Clone)]
pub struct Ingredient
{
    pub amount: u8,                 // in ml
    pub name: String
}

#[derive(Debug)]
pub struct Cocktail
{
    pub name: String,
    pub glass: String,
    pub category: String,
    pub ingredients: Vec<Ingredient>,
    pub garnish: String,
	pub shaken_not_stirred: Option<bool>,
    pub preparation: String
}

impl Default for Cocktail
{
    fn default() -> Cocktail
    {
        Cocktail
        {
            name: String::from("Empty Glass"),
            glass: String::from("Any"),
            category: String::from("All Day"),
    		shaken_not_stirred: None,
            ingredients: Vec::new(),
            garnish: String::from("Slice of thin air"),
            preparation: String::from("Pour nothing into a glass of your choice.")
        }
    }
}

pub fn convert_measure(measure: &str) -> Result<u8, Box<dyn Error>>
{
    // a measure consists of a whole number optionally a fraction and a unit. Between whole
    // number and fraction a whitespace is required Between fraction and unit it's optional.
    // at one of whole number and fraction and the unit are required.
    // examples:
    //      1 1/2 oz
    //        1/3 oz
    //      5 oz
    // spaces before / after the slash are not allowed
    // (?:) denotes a non-capturing group
    // (?P<name>) denotes a named capure group
    //
    // the regex is:
    //  ^\s*(?:(?P<whole>\d+){0,1}\s+){0,1}(?:(?P<numerator>\d+)/(?P<denominator>\d+)){0,1}\s*(?P<unit>\w+)
    //
    // let expr = Regex::new(r"^\s*(?:(?P<whole>\d+){0,1}\s+){0,1}(?:(?P<numerator>\d+)/(?P<denominator>\d+)){0,1}\s*(?P<unit>\w+)").unwrap();
    let expr = Regex::new(r"^\s*(?:(?P<whole>\d+(?:\.\d+){0,1})\s+){0,1}(?:(?P<numerator>\d+)/(?P<denominator>\d+)){0,1}\s*(?P<unit>[a-z]+){0,1}")?;

    let captures = expr.captures(measure).unwrap();     // match regex

    let mut value: Option<f64> = None;

    if captures.name("whole").is_some()                 // check if capture group "whole" has matched
    {
        // add the whole number part to value
        value = Some(captures.name("whole").unwrap().as_str().parse::<f64>().unwrap());
    };

    // check if there is a fraction (numerator and denominator)
    if captures.name("numerator").is_some() && captures.name("denominator").is_some()
    {
        // get numerator's and denominator's value
        let numerator = captures.name("numerator").unwrap().as_str().parse::<f64>().unwrap();
        let denominator = captures.name("denominator").unwrap().as_str().parse::<f64>().unwrap();

        if denominator > 0.0                    // don't divde by zero
        {
            value = Some(value.unwrap_or(0.0) + numerator / denominator);   // add the value of the fractional part
        };
    };

    let mut conversion: f64 = 0.0;              // the conversion factor from unit to ml

    if captures.name("unit").is_some()          // check if there is a match for unit
    {
        conversion = match captures.name("unit").unwrap().as_str().to_lowercase().as_str()
        {
            "oz" => 30.0,
            "ml" => 1.0,
            "cl" => 10.0,
            "dash" => 1.0,
            "tl" => 5.0,
            "bl" => 2.0,
            "part" => 40.0,
            "el" => 15.0,
            "shot" => 45.0,
            _ => panic!(), //0.0,
        };

            /*
                1 oz = 30 ml,
                1 ml = 1 ml,
                1 cl = 10 ml,
                1 l = 1000 ml,
                1 Dash = 1 ml,
                1 Tl = 5 ml,
                1 Bl = 2 ml,
                1 Part = 40 ml,
                1 Jigger = 42 ml,
                1 El = 15 ml,
                1 Gill = 36 ml,
                1 Pony = 30 ml,
                1 Schuss = 1 ml,
                1 Short = 20 ml,
                1 Shot = 45 ml

            */
    };

    if value.is_some() && value.unwrap() <= (u8::max_value() as f64) // is the value valid?
    {
        Ok(((value.unwrap() * conversion).round()) as u8)            // converted to ml and return
    }
    else if value.is_none()
    {
        Ok(conversion.round() as u8)                    // unit without value is trated as
                                                        // one unit e. g. "oz" -> 1 oz = 30 ml
                                                        // returns 30
    }
    else
    {
        Err(Box::new(ConversionError::ValueToBig))    // value > u8:max_value
    }
}

#[cfg(test)]
mod tests {
     use super::*;

    #[test]
    fn number_with_fraction()
    {
        assert_eq!(convert_measure("1 1/2 oz"), 45);
    }
    #[test]
    fn fraction_only()
    {
        assert_eq!(convert_measure("1/2 oz"), 15);
    }
    #[test]
    fn whole_number()
    {
        assert_eq!(convert_measure("3 oz"), 90);
    }
    #[test]
    fn decimal()
    {
        assert_eq!(convert_measure("1.5 oz"), 45);
    }
    #[test]
    fn whitespace()
    {
        assert_eq!(convert_measure("1       oz"), 30);
    }
    #[test]
    #[ignore]
    fn no_whitespace()
    {// currently not implemented
        assert_eq!(convert_measure("1oz"), 30);
    }
    #[test]
        fn unreduced()
    {
        assert_eq!(convert_measure("30/60 oz"), 15);
    }
    #[test]
    fn overflow()
    {
        convert_measure("100 oz");
    }
    #[test]
    #[should_panic]
    fn unknown()
    {
        convert_measure("1.5 l");
    }
    #[test]
    fn measure_only()
    {
        convert_measure("oz");
    }
}
