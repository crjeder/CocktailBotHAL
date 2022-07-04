use crate::cocktail::{GenericCocktail, Ingredient};

enum TestCoctails
{
    Vesper, Margarita, Screwdriver, Sunrise
}

fn test_cocktail(drink: TestCoctails) -> GenericCocktail
{
    match drink
    {
        Vesper => GenericCocktail
            {
                name: String::from("Vesper"),
                glass: String::from("martini"),
                category: Some(String::from("Before Dinner Cocktail")),
                ingredients: vec!
                [
                    Ingredient {amount: 60, name: String::from("Gin")},
                    Ingredient {amount: 15, name: String::from("Vodka")},
                    Ingredient {amount: 8, name: String::from("Lillet Blonde")}
                ],
                garnish: Some(String::from("Lemon twist")),
                preparation: Some(Shaken),
                preparation_instructions: Some(String::from("Shake and strain into a chilled cocktail glass."))
            },
        Margarita => GenericCocktail
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
                ],
                garnish: None,
                preparation: Some(Shaken),
                preparation_instructions: Some(String::from(
                    "Rub the rim of the glass with the lime slice to make the salt stick to it. \
                    Take care to moisten only the outer rim and sprinkle the salt on it. \
                    The salt should present to the lips of the imbiber and never mix into the cocktail. \
                    Shake the other ingredients with ice, then carefully pour into the glass."))
            },
        Screwdriver => GenericCocktail
            {
                name: String::from("Screwdriver"),
                glass: String::from("Highball"),
                category: Some(String::from("All Day")),
    		    ingredients: vec!
                [
                    Ingredient {amount: 20, name: String::from("Vodka")},
                    Ingredient {amount: 80, name: String::from("Orange Juice")}
                ],
                preparation: Some(Shaken),
                garnish: Some(String::from("Slice of Orange")),
                preparation_instructions: Some(String::from("Pour Vodka and Orange Juice over ice"))
            },
    	Sunrise => GenericCocktail
            {
                name: String::from("Tequila Sunrise"),
                glass: String::from("Huricane"),
                category: Some(String::from("Classic")),
    		    ingredients: vec!
                [
                    Ingredient {amount: 10, name: String::from("Grenadine")},
    		        Ingredient {amount: 60, name: String::from("Tequila")},
                    Ingredient {amount: 15, name: String::from("Tripple Sec")},
    		        Ingredient {amount: 75, name: String::from("Orange Juice")},
    		        Ingredient {amount: 20, name: String::from("Lime Juice")}
                ],
                garnish: Some(String::from("Slice of orange and cherry")),
                preparation: Some(Shaken),
                preparation: Some(String::from("Pour Grenadine over ice and then add the other ingredient slowly. Do not stir!"))
            }
        //_ => {GenericCocktail::default()}
    }
}
