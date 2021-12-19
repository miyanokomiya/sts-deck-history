use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct DeckDiff {
    pub floor: i32,
    pub obtained: Vec<String>,
    pub removed: Vec<String>,
    pub transformed: Vec<String>,
    pub upgraded: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct MasterDeck {
    pub cards: Vec<String>,
    pub unknown_obtained: Vec<String>,
    pub unknown_removed: Vec<String>,
}

impl MasterDeck {
    pub fn new(cards: Vec<String>) -> MasterDeck {
        MasterDeck {
            cards,
            unknown_obtained: vec![],
            unknown_removed: vec![],
        }
    }

    pub fn obtain(&mut self, card: String) {
        self.cards.push(card);
    }

    pub fn remove(&mut self, card: String) {
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => {
                self.cards.remove(index);
            }
            _ => {
                self.unknown_obtained.push(card);
            }
        }
    }

    pub fn upgrade(&mut self, card: String) {
        let upgraded = upgrade_card(card.clone());
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => self.cards[index] = upgraded,
            _ => {
                self.unknown_obtained.push(card);
                self.obtain(upgraded);
            }
        }
    }

    pub fn downgrade(&mut self, card: String) {
        let downgraded = downgrade_card(card.clone());
        match self.cards.iter().position(|c| *c == card) {
            Some(index) => self.cards[index] = downgraded,
            _ => {
                self.unknown_obtained.push(card);
                self.obtain(downgraded);
            }
        }
    }

    pub fn merge_at_last(&mut self, last_cards: &Vec<String>) {
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

    pub fn redo(&mut self, diff: &DeckDiff) {
        diff.obtained.iter().for_each(|c| self.obtain((*c).clone()));
        diff.upgraded
            .iter()
            .for_each(|c| self.upgrade((*c).clone()));
        diff.removed.iter().for_each(|c| self.remove((*c).clone()));
        diff.transformed
            .iter()
            .for_each(|c| self.remove((*c).clone()));
    }

    pub fn undo(&mut self, diff: &DeckDiff) {
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
