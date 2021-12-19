mod master_deck;
mod resource;

use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use master_deck::{DeckDiff, MasterDeck};
use resource::Resource;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    file: String,
}

#[derive(Deserialize, Debug)]
struct RunData {
    master_deck: Vec<String>,
    card_choices: Option<Vec<CardChoice>>,
    event_choices: Option<Vec<EventChoice>>,
    items_purged: Option<Vec<String>>,
    items_purged_floors: Option<Vec<i32>>,
    // including relics and potions
    items_purchased: Option<Vec<String>>,
    item_purchase_floors: Option<Vec<i32>>,
    campfire_choices: Option<Vec<CampfireChoice>>,
    floor_reached: i32,
    character_chosen: String,
}

#[derive(Deserialize, Debug)]
struct CardChoice {
    floor: i32,
    picked: String,
}

#[derive(Deserialize, Debug)]
struct EventChoice {
    floor: i32,
    cards_obtained: Option<Vec<String>>,
    cards_removed: Option<Vec<String>>,
    cards_transformed: Option<Vec<String>>,
    cards_upgraded: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct CampfireChoice {
    floor: i32,
    key: String, // "PURGE" | "SMITH" | etc...
    data: Option<String>,
}

fn main() {
    let args = Args::parse();

    let file = File::open(args.file).unwrap();
    let reader = BufReader::new(file);
    let deserialized: RunData = serde_json::from_reader(reader).unwrap();

    let mut card_choice_map: HashMap<i32, Vec<CardChoice>> = HashMap::new();
    match deserialized.card_choices {
        Some(ccs) => {
            for cc in ccs {
                card_choice_map.entry(cc.floor).or_insert(vec![]).push(cc);
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

    let mut items_purged_map: HashMap<i32, Vec<String>> = HashMap::new();
    match deserialized.items_purged {
        Some(ips) => {
            let ipfs = deserialized.items_purged_floors.unwrap();
            for (i, ip) in ips.iter().enumerate() {
                items_purged_map
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

    let mut event_choice_map: HashMap<i32, EventChoice> = HashMap::new();
    match deserialized.event_choices {
        Some(ecs) => {
            for ec in ecs {
                event_choice_map.insert(ec.floor, ec);
            }
        }
        _ => (),
    }

    let mut deck_diff: Vec<DeckDiff> = vec![];
    for floor in 0..deserialized.floor_reached {
        match get_current_floor_diff(
            &card_choice_map,
            &items_purchased_map,
            &items_purged_map,
            &campfire_choice_map,
            &event_choice_map,
            floor,
        ) {
            Some(res) => deck_diff.push(res),
            _ => (),
        }
    }

    let mut master_deck = MasterDeck::new(Resource::get_origin_deck(deserialized.character_chosen));
    deck_diff.iter().for_each(|d| master_deck.redo(d));
    master_deck.merge_at_last(&deserialized.master_deck);

    println!("last: {:?}", master_deck.cards);
    println!("obtained?: {:?}", master_deck.unknown_obtained);
    println!("removed?: {:?}", master_deck.unknown_removed);
}

const PICK_SKIP: &str = "SKIP";
const KEY_SMITH: &str = "SMITH";
const KEY_PURGE: &str = "PURGE";

fn get_current_floor_diff(
    card_choice_map: &HashMap<i32, Vec<CardChoice>>,
    items_purchased_map: &HashMap<i32, Vec<String>>,
    items_purged_map: &HashMap<i32, Vec<String>>,
    campfire_choice_map: &HashMap<i32, CampfireChoice>,
    event_choice_map: &HashMap<i32, EventChoice>,
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
        Some(ccs) => {
            for cc in ccs {
                if cc.picked != PICK_SKIP {
                    diff.obtained.push(cc.picked.clone());
                }
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

    match items_purged_map.get(&floor) {
        Some(ip) => {
            for val in ip {
                diff.removed.push(val.clone());
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

    match event_choice_map.get(&floor) {
        Some(ec) => {
            match ec.cards_obtained.clone() {
                Some(values) => {
                    for v in values {
                        diff.obtained.push(v);
                    }
                }
                _ => (),
            }
            match ec.cards_removed.clone() {
                Some(values) => {
                    for v in values {
                        diff.removed.push(v);
                    }
                }
                _ => (),
            }
            match ec.cards_transformed.clone() {
                Some(values) => {
                    for v in values {
                        diff.transformed.push(v);
                    }
                }
                _ => (),
            }
            match ec.cards_upgraded.clone() {
                Some(values) => {
                    for v in values {
                        diff.upgraded.push(v);
                    }
                }
                _ => (),
            }
        }
        _ => (),
    }

    if diff.obtained.len() > 0
        || diff.removed.len() > 0
        || diff.transformed.len() > 0
        || diff.upgraded.len() > 0
    {
        Some(diff)
    } else {
        None
    }
}
