
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};

use serde::{Deserialize, Serialize};

/// REST API
///
/// base URL:  http://localhost:8000
///
/// get API documentation
/// http://localhost:8000/
#[get("/")]
fn index() -> &'static str { "Hello, world!" }

/// get the liquids the dispenser has loaded
/// http://localhost:8000/REST/get/liquids
#[get("/REST/get/liquids")]
fn get_liquids(l: State<Cocktailbot>) -> Option<Json<&Vec<Liquid>>>
{
    Some(Json(&l.inner().dispenser.liquids))
}

/// get the glasses the bar bot knows abaout
/// http://localhost:8000/REST/get/glasses
#[get("/REST/get/glasses")]
fn get_glasses(l: State<Cocktailbot>) -> Option<Json<&Vec<Glass>>>
{
    Some(Json(&l.inner().glasses))
}

/// have a drink mixed
/// http://localhost:8000/REST/post/cocktail
#[post("/REST/post/cocktail", data = "<recipe>")]
//fn mix_cocktail(bot: State<Cocktailbot>, recipe: Cocktail) -> Option<Json<BarBotError>>
fn mix_cocktail(
    bot: State<Mutex<Cocktailbot>>,
    recipe: Json<GenericCocktail>,
) -> JsonValue
{
    let my_bot = &mut bot.inner().lock().unwrap();
    my_bot.cocktails_mixed += 1;

    json!
    ({
        "status": "success",
        "cocktail": recipe.name,
        "cocktails_mixed": my_bot.cocktails_mixed
    })
}

fn main() -> std::io::Result<()>
{
    let this_bot: Mutex<Cocktailbot> = Mutex::new(Cocktailbot {
        config: Config { display: false },
        dispenser: Dispenser {
            max_liquids: 16,
            liquids: vec![
                // density in g / 100 ml, alcohol in vol % = g / 100 ml, suggar g / 100 g (or 100 ml)
                Liquid {
                    name: String::from("Vodka"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Gin"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("White Rum"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Tequila"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Tripple Sec"),
                    density: 94,
                    alcohol: 40,
                    suggar: 10,
                },
                Liquid {
                    name: String::from("Orange Juice"),
                    density: 104,
                    alcohol: 0,
                    suggar: 9,
                },
                Liquid {
                    name: String::from("Ginger Ale"),
                    density: 104,
                    alcohol: 40,
                    suggar: 3,
                },
                Liquid {
                    name: String::from("Tonic"),
                    density: 104,
                    alcohol: 40,
                    suggar: 8,
                },
                Liquid {
                    name: String::from("Lime Juice"),
                    density: 104,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Coke"),
                    density: 104,
                    alcohol: 0,
                    suggar: 9,
                },
                Liquid {
                    name: String::from("Prosecco"),
                    density: 94,
                    alcohol: 40,
                    suggar: 0,
                },
                Liquid {
                    name: String::from("Grenadine"),
                    density: 121,
                    alcohol: 0,
                    suggar: 47,
                },
                Liquid {
                    name: String::from("Aperol"),
                    density: 100,
                    alcohol: 40,
                    suggar: 47,
                },
                Liquid {
                    name: String::from("Syrup"),
                    density: 129,
                    alcohol: 0,
                    suggar: 60,
                },
                Liquid {
                    name: String::from("Amaretto"),
                    density: 100,
                    alcohol: 40,
                    suggar: 47,
                },
            ],
        },
        glasses: vec![
            Glass {
                name: String::from("Highball"),
                volume: 100,
                weight: 80,
            },
            Glass {
                name: String::from("Tumbler"),
                volume: 100,
                weight: 50,
            },
        ],
        cocktails_mixed: 0,
    });

    let _rocket = rocket::ignite()
        .mount("/", routes![index, get_liquids, get_glasses, mix_cocktail])
        .manage(this_bot)
        .launch();

    Ok(())
}
