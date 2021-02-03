fn main()
{
    let screwdriver = Cocktail
    {
        name: String::from("Screwdriver"),
        glass: String::from("Highball"),
        category: String::from("All Day"),
        ingredients: vec!
        [
          Ingredient {amount: 20, name: String::from("Vodka")},
          Ingredient {amount: 80, name: String::from("Orange Juice")}
        ],
        garnish: String::from("Slice of Orange"),
        preparation: String::from("Pour Vodka and Orange Juice over ice")
    };

    println!("Your Cocktail Choice is:");
    println!("{}", screwdriver.name);
    println!("Ingredients:");
    for liquid in screwdriver.ingredients.iter()
    {
        println!("{} ml {}", liquid.amount, liquid.name);
    }
    println!("{}", screwdriver.preparation);
    println!("Garnish with {}", screwdriver.garnish);
    println!("Enjoy!");

    let config = Config {dispensers: 16, display: false};
    let dispenser_config = vec!
    [
        Liquid {name: String::from("Vodka"), density: 94, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Gin"), density: 94, alcohol: 40, suggar: 0},
        Liquid {name: String::from("White Rum"), density: 94, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Tequila"), density: 94, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Tripple Sec"), density: 94, alcohol: 40, suggar: 10},
        Liquid {name: String::from("Orange Juice"), density: 104, alcohol: 0, suggar: 8},
        Liquid {name: String::from("Ginger Ale"), density: 104, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Bitter Lemon"), density: 104, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Lime Juice"), density: 104, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Coke"), density: 104, alcohol: 0, suggar: 0},
        Liquid {name: String::from("Prosecco"), density: 94, alcohol: 40, suggar: 0},
        Liquid {name: String::from("Grenadine"), density: 121, alcohol: 0, suggar: 47},
        Liquid {name: String::from("Aperol"), density: 100, alcohol: 40, suggar: 47},
        Liquid {name: String::from("Syrup"), density: 121, alcohol: 0, suggar: 60},
        Liquid {name: String::from("Amaretto"), density: 100, alcohol: 40, suggar: 47}
    ];

    let glasses = vec!
    [
        Glass {name: String::from("Highball"), volume: 100, weight: 80},
        Glass {name: String::from("Tumbler"), volume: 100, weight: 80}
    ];
}

struct Ingredient
{
    amount: u8,                 // in ml
    name: String
}

struct Cocktail
{
    name: String,
    glass: String,
    category: String,
    ingredients: Vec<Ingredient>,
    garnish: String,
    preparation: String
}

struct Config
{
    dispensers: u8,
    display: bool
}

struct Glass
{
    name: String,
    volume: u8,                 // in ml
    weight: u8                  // in g
}

struct Liquid
{
    name: String,
    density: u8,                // in g / cm3 * 100
    alcohol: u8,                // in vol % (ABV)
    suggar: u8,                 // in g / 100 g
}
