mod resource;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use resource::Resource;

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
    // including relics and potions
    items_purchased: Option<Vec<String>>,
    item_purchase_floors: Option<Vec<i32>>,
    campfire_choices: Option<Vec<CampfireChoice>>,
    floor_reached: i32,
    character_chosen: String,
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

#[derive(Serialize, Deserialize, Debug)]
struct MasterDeck {
    cards: Vec<String>,
    unknown_obtained: Vec<String>,
    unknown_removed: Vec<String>,
}

impl MasterDeck {
    fn new(cards: Vec<String>) -> MasterDeck {
        MasterDeck {
            cards,
            unknown_obtained: vec![],
            unknown_removed: vec![],
        }
    }

    fn obtain(&mut self, card: String) {
        self.cards.push(card);
    }

    fn remove(&mut self, card: String) {
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => {
                self.cards.remove(index);
            }
            _ => {
                self.unknown_obtained.push(card);
            }
        }
    }

    fn upgrade(&mut self, card: String) {
        let upgraded = upgrade_card(card.clone());
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => self.cards[index] = upgraded,
            _ => {
                self.unknown_obtained.push(card);
                self.obtain(upgraded);
            }
        }
    }

    fn downgrade(&mut self, card: String) {
        let downgraded = downgrade_card(card.clone());
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => self.cards[index] = downgraded,
            _ => {
                self.unknown_obtained.push(card);
                self.obtain(downgraded);
            }
        }
    }

    fn merge_at_last(&mut self, last_cards: &Vec<String>) {
        let mut current = self.cards.clone();
        last_cards
            .iter()
            .for_each(|lc| match current.iter().position(|c| c == lc) {
                Some(index) => {
                    current.remove(index);
                }
                _ => {
                    self.unknown_obtained.push(lc.clone());
                    self.obtain(lc.clone());
                }
            });

        current.iter().for_each(|c| {
            self.unknown_removed.push(c.clone());
            self.remove(c.clone());
        });
    }

    fn redo(&mut self, diff: &DeckDiff) {
        diff.obtained.iter().for_each(|c| self.obtain((*c).clone()));
        diff.upgraded
            .iter()
            .for_each(|c| self.upgrade((*c).clone()));
        diff.removed.iter().for_each(|c| self.remove((*c).clone()));
        diff.transformed
            .iter()
            .for_each(|c| self.remove((*c).clone()));
    }

    fn undo(&mut self, diff: &DeckDiff) {
        diff.transformed
            .iter()
            .for_each(|c| self.obtain((*c).clone()));
        diff.removed.iter().for_each(|c| self.obtain((*c).clone()));
        diff.upgraded
            .iter()
            .for_each(|c| self.downgrade((*c).clone()));
        diff.obtained.iter().for_each(|c| self.remove((*c).clone()));
    }
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
        match reset_current_floor(
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

    // for diff in deck_diff {
    //     println!("{:?}", diff);
    // }

    let mut master_deck = MasterDeck::new(Resource::get_origin_deck(deserialized.character_chosen));
    // deck_diff.reverse();
    deck_diff.iter().for_each(|d| master_deck.redo(d));
    master_deck.merge_at_last(&deserialized.master_deck);
    println!("last: {:?}", master_deck.cards);
    println!("obtained?: {:?}", master_deck.unknown_obtained);
    println!("removed?: {:?}", master_deck.unknown_removed);
}

const PICK_SKIP: &str = "SKIP";
const KEY_SMITH: &str = "SMITH";
const KEY_PURGE: &str = "PURGE";

fn reset_current_floor(
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

fn get_card_upgraded(card: String) -> (String, i32) {
    let splited: Vec<&str> = card.split("+").collect();
    if splited.len() == 0 {
        (card, 0)
    } else {
        match splited.last().unwrap().parse::<i32>() {
            Ok(level) => (splited.first().unwrap().to_string(), level),
            _ => (card, 0),
        }
    }
}

fn upgrade_card(card: String) -> String {
    let (name, level) = get_card_upgraded(card);
    name + "+" + &(level + 1).to_string()
}

fn downgrade_card(card: String) -> String {
    let (name, level) = get_card_upgraded(card);
    if level > 1 {
        name + "+" + &(level - 1).to_string()
    } else {
        name
    }
}
