//! Fun, cartoonish objects and animals for the game.

pub const ANIMALS: &[&str] = &[
    "A disco-dancing penguin",
    "A space-traveling hamster",
    "A surfing giraffe",
    "A monocle-wearing octopus",
    "A skateboarding bulldog",
    "A wizard cat",
    "A weightlifting bunny",
    "A scuba-diving elephant",
    "A jetpack-wearing sloth",
    "A breakdancing turtle",
];

pub const OBJECTS: &[&str] = &[
    "A giant floating taco",
    "A sentient toaster",
    "A rocket-powered unicycle",
    "A crystal ball with a smiley face",
    "A rubber ducky with a crown",
    "A marshmallow castle",
    "A flying pizza slice",
    "A neon-glowing boombox",
    "A teapot that breathes bubbles",
    "A pair of sneakers with wings",
];

pub const LOCATIONS: &[&str] = &[
    "in outer space",
    "on a tropical beach",
    "inside a giant candy bowl",
    "on top of a snowy mountain",
    "under the ocean",
    "in a futuristic neon city",
    "in a magical forest",
    "on a floating island",
    "at a robot disco",
    "inside a giant bubble",
];

use rand::seq::SliceRandom;

/// Generate a random composite goal and individual starting objects for players.
pub fn generate_game_assets(player_count: usize) -> (String, Vec<String>) {
    let mut rng = rand::thread_rng();
    
    // Pick 2-3 random elements for the composite goal
    let animal = ANIMALS.choose(&mut rng).unwrap_or(&"A mystery animal");
    let object = OBJECTS.choose(&mut rng).unwrap_or(&"A mystery object");
    let location = LOCATIONS.choose(&mut rng).unwrap_or(&"in a mystery place");
    
    let communal_goal = format!("{} with {} {}", animal, object, location);
    
    // Pick unique starting objects for each player
    let mut all_options = [ANIMALS, OBJECTS].concat();
    all_options.shuffle(&mut rng);
    
    let player_objects = all_options.into_iter()
        .take(player_count)
        .map(|s| s.to_string())
        .collect();
        
    (communal_goal, player_objects)
}
