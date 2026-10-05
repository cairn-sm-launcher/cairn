//! Le nom de mode montrable.
//!
//! L'API donne le nom du fichier de script : `Obstacle.Script.txt`,
//! `SM_Elite_Online.Script.txt`. Ce n'est pas un libelle qu'on affiche dans
//! une liste, et c'est pourtant sur lui que repose le filtre par mode.

/// Tire un libelle du nom de fichier de script.
///
/// Retire l'extension, le prefixe de jeu, le suffixe `_Online`, et rend les
/// soulignements sous forme d'espaces. Un script sans nom ou entierement
/// mange par ces regles devient `Custom`, qui est ce que le joueur comprend :
/// un mode qui n'est pas l'un des modes connus.
pub fn lisible(script: &str) -> String {
    let mut m = script.trim();

    // L'extension, insensible a la casse : l'API n'est pas regulière.
    for extension in [".Script.txt", ".script.txt", ".Script.Txt"] {
        if let Some(sans) = m.strip_suffix(extension) {
            m = sans;
            break;
        }
    }

    for prefixe in ["SM_", "TM_", "sm_", "tm_"] {
        if let Some(sans) = m.strip_prefix(prefixe) {
            m = sans;
            break;
        }
    }

    for suffixe in ["_Online", "_online"] {
        if let Some(sans) = m.strip_suffix(suffixe) {
            m = sans;
            break;
        }
    }

    let libelle = m.replace('_', " ");
    let libelle = libelle.trim();

    if libelle.is_empty() {
        "Custom".to_string()
    } else {
        libelle.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_scripts_reels_de_shootmania() {
        assert_eq!(lisible("Obstacle.Script.txt"), "Obstacle");
        assert_eq!(lisible("SM_Elite_Online.Script.txt"), "Elite");
        assert_eq!(lisible("SM_Royal_Online.Script.txt"), "Royal");
        assert_eq!(lisible("SM_Siege_Online.Script.txt"), "Siege");
        assert_eq!(lisible("SM_Battle_Online.Script.txt"), "Battle");
        assert_eq!(lisible("SM_Joust_Online.Script.txt"), "Joust");
    }

    #[test]
    fn les_soulignements_internes_deviennent_des_espaces() {
        assert_eq!(lisible("SM_Combo_Team_Online.Script.txt"), "Combo Team");
    }

    #[test]
    fn un_script_absent_ou_vide_donne_custom() {
        assert_eq!(lisible(""), "Custom");
        assert_eq!(lisible("   "), "Custom");
        assert_eq!(lisible("SM_.Script.txt"), "Custom");
    }

    #[test]
    fn un_script_deja_propre_traverse_sans_dommage() {
        assert_eq!(lisible("Obstacle"), "Obstacle");
    }

    #[test]
    fn le_prefixe_n_est_retire_qu_une_fois() {
        // Un mode qui s'appellerait vraiment "SM Quelque chose" garderait
        // son second prefixe, ce qui est preferable a le raboter deux fois.
        assert_eq!(lisible("SM_SM_Test.Script.txt"), "SM Test");
    }
}
