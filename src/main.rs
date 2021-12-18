use clap::Parser;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    file: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RunData {
    master_deck: Vec<String>,
    card_choices: Option<Vec<CardChoice>>,
    event_choices: Option<Vec<EventChoice>>,
    items_purged: Option<Vec<String>>,
    items_purged_floors: Option<Vec<i32>>,
    items_purchased: Option<Vec<String>>,
    item_purchase_floors: Option<Vec<i32>>,
    campfire_choices: Option<Vec<CampfireChoice>>,
    floor_reached: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CardChoice {
    floor: i32,
    picked: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EventChoice {
    floor: i32,
    cards_obtained: Option<Vec<String>>,
    cards_removed: Option<Vec<String>>,
    cards_transformed: Option<Vec<String>>,
    cards_upgraded: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CampfireChoice {
    floor: i32,
    key: String, // "PURGE" | "SMITH" | etc...
    data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeckDiff {
    floor: i32,
    obtained: Vec<String>,
    removed: Vec<String>,
    transformed: Vec<String>,
    upgraded: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let file = File::open(args.file).unwrap();
    let reader = BufReader::new(file);

    let deserialized: RunData = serde_json::from_reader(reader).unwrap();

    let mut card_choice_map: HashMap<i32, CardChoice> = HashMap::new();
    match deserialized.card_choices {
        Some(ccs) => {
            for cc in ccs {
                card_choice_map.insert(cc.floor, cc);
            }
        }
        _ => (),
    }

    let mut items_purchased_map: HashMap<i32, Vec<String>> = HashMap::new();
    match deserialized.items_purchased {
        Some(ips) => {
            let ipfs = deserialized.item_purchase_floors.unwrap();
            for (i, ip) in ips.iter().enumerate() {
                items_purchased_map
                    .entry(ipfs[i])
                    .or_insert(vec![])
                    .push(ip.clone());
            }
        }
        _ => (),
    }

    let mut campfire_choice_map: HashMap<i32, CampfireChoice> = HashMap::new();
    match deserialized.campfire_choices {
        Some(cfcs) => {
            for cfc in cfcs {
                campfire_choice_map.insert(cfc.floor, cfc);
            }
        }
        _ => (),
    }

    let mut deck_diff: Vec<DeckDiff> = vec![];
    for floor in 0..deserialized.floor_reached {
        match reset_current_floor(
            &card_choice_map,
            &items_purchased_map,
            &campfire_choice_map,
            floor,
        ) {
            Some(res) => deck_diff.push(res),
            _ => (),
        }
    }

    for diff in deck_diff {
        println!("{:?}", diff);
    }
}

const PICK_SKIP: &str = "SKIP";
const KEY_SMITH: &str = "SMITH";
const KEY_PURGE: &str = "PURGE";

fn reset_current_floor(
    card_choice_map: &HashMap<i32, CardChoice>,
    items_purchased_map: &HashMap<i32, Vec<String>>,
    campfire_choice_map: &HashMap<i32, CampfireChoice>,
    floor: i32,
) -> Option<DeckDiff> {
    let mut diff = DeckDiff {
        floor,
        obtained: vec![],
        removed: vec![],
        transformed: vec![],
        upgraded: vec![],
    };

    match card_choice_map.get(&floor) {
        Some(cc) => {
            if cc.picked != PICK_SKIP {
                diff.obtained.push(cc.picked.clone());
            }
        }
        _ => (),
    }

    match items_purchased_map.get(&floor) {
        Some(ip) => {
            for val in ip {
                diff.obtained.push(val.clone());
            }
        }
        _ => (),
    }

    match campfire_choice_map.get(&floor) {
        Some(cfc) => {
            if cfc.key == KEY_SMITH {
                diff.upgraded.push(cfc.data.clone().unwrap());
            } else if cfc.key == KEY_PURGE {
                diff.removed.push(cfc.data.clone().unwrap());
            }
        }
        _ => (),
    }

    if diff.obtained.len() > 0 || diff.removed.len() > 0 {
        Some(diff)
    } else {
        None
    }
}
