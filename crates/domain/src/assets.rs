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
    "A karate-chopping kangaroo",
    "A DJ-ing dolphin",
    "A detective owl",
    "A chef raccoon",
    "A ballerina hippo",
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
    "A golden banana trophy",
    "A hoverboard made of cookies",
    "A magic wand that shoots confetti",
    "A backpack full of rainbows",
    "A telescope that sees into the future",
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
    "at a dinosaur tea party",
    "on a cloud made of cotton candy",
    "inside a giant clock",
    "at a carnival for aliens",
    "in a library of floating books",
];

pub const MODIFIERS: &[&str] = &[
    "wearing a top hat",
    "holding a lightsaber",
    "wearing sunglasses",
    "riding a skateboard",
    "eating a pizza",
    "on fire (safely)",
    "covered in glitter",
    "wearing a cape",
    "holding a balloon",
    "wearing clown shoes",
    "surrounded by butterflies",
    "holding a sign that says 'Help'",
    "wearing a tutu",
    "holding a rubber chicken",
    "wearing a space helmet",
    "that is giant",
    "that is tiny",
    "that is glowing green",
    "that is invisible (mostly)",
    "made of jelly",
];

use rand::seq::SliceRandom;

/// Generate a random composite goal and individual starting objects for players.
pub fn generate_game_assets(player_count: usize) -> (String, Vec<String>) {
    let mut rng = rand::thread_rng();
    
    // Pick 2-3 random elements for the composite goal
    let animal = ANIMALS.choose(&mut rng).unwrap_or(&"A mystery animal");
    let object = OBJECTS.choose(&mut rng).unwrap_or(&"A mystery object");
    let location = LOCATIONS.choose(&mut rng).unwrap_or(&"in a mystery place");
    
    // Create a more complex composite goal
    let communal_goal = format!("{} holding {} {}", animal, object, location);
    
    // Pick unique starting objects for each player
    // We want these to be simple (just one object or animal)
    let mut all_options = [ANIMALS, OBJECTS].concat();
    all_options.shuffle(&mut rng);
    
    let player_objects = all_options.into_iter()
        .take(player_count)
        .map(|s| s.to_string())
        .collect();
        
    (communal_goal, player_objects)
}

/// Generate 4 random modification options.
pub fn generate_modification_options() -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut options: Vec<String> = MODIFIERS.iter().map(|s| s.to_string()).collect();
    options.shuffle(&mut rng);
    options.into_iter().take(4).collect()
}

/// Apply a modification to an object description.
pub fn apply_modification(object: &str, modifier: &str) -> String {
    format!("{} {}", object, modifier)
}
