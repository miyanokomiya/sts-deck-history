pub struct Resource {}

impl Resource {
    pub fn get_origin_deck(character_chosen: String) -> Vec<String> {
        println!("Character: {:?}", character_chosen);
        if character_chosen == "IRONCLAD" {
            vec![
                "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Strike_R", "Defend_R", "Defend_R",
                "Defend_R", "Defend_R", "Bash",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect()
        } else {
            vec![]
        }
    }
}
