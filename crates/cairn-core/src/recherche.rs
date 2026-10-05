//! La recherche dans la liste des serveurs.
//!
//! Deux exigences qui se contredisent a peine. Elle doit etre tolerante,
//! parce qu'un joueur tape « atria » et pas « Λtría - Stretching ». Et
//! elle doit rester previsible, parce qu'une recherche dont on ne
//! comprend pas les resultats fait douter de la liste entiere.
//!
//! Le compromis retenu : on normalise des deux cotes, puis **chaque mot
//! de la requete doit se trouver quelque part**. Pas de distance
//! d'edition, pas de score. « atria ob » trouve « Λtría - Obstacle »,
//! « obstacle atria » le trouve aussi, et rien ne remonte sans raison
//! visible.
//!
//! La recherche porte sur le nom **debarrasse de ses codes**, sinon un
//! `$` tape par erreur casserait la frappe, et sur le login, parce que
//! c'est parfois lui qu'on connait.

/// Ramene une chaine a sa forme comparable : minuscules, sans accents,
/// sans les ligatures et les lettres decoratives courantes.
///
/// La table est ecrite a la main plutot que tiree d'une bibliotheque de
/// normalisation Unicode. Elle couvre le latin etendu, qui est ce que
/// portent les noms de serveurs, et elle evite une dependance de plus
/// dans une caisse qui n'en a qu'une.
pub fn normaliser(valeur: &str) -> String {
    let mut sortie = String::with_capacity(valeur.len());
    for c in valeur.chars().flat_map(|c| c.to_lowercase()) {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'α' | 'λ' => {
                sortie.push('a')
            }
            'ç' | 'ć' | 'č' => sortie.push('c'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => sortie.push('e'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' => sortie.push('i'),
            'ñ' | 'ń' | 'ň' => sortie.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => sortie.push('o'),
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => sortie.push('u'),
            'ý' | 'ÿ' => sortie.push('y'),
            'š' | 'ś' => sortie.push('s'),
            'ž' | 'ź' | 'ż' => sortie.push('z'),
            'ð' | 'ď' => sortie.push('d'),
            'ł' => sortie.push('l'),
            'ť' => sortie.push('t'),
            'ř' => sortie.push('r'),
            'æ' => sortie.push_str("ae"),
            'œ' => sortie.push_str("oe"),
            'ß' => sortie.push_str("ss"),
            _ => sortie.push(c),
        }
    }
    sortie
}

/// Vrai quand chaque mot de la requete se trouve dans l'un des champs.
///
/// Une requete vide accepte tout : c'est l'etat au repos de la barre de
/// recherche, elle ne doit rien filtrer.
pub fn correspond(champs: &[&str], requete: &str) -> bool {
    let mots: Vec<String> = requete
        .split_whitespace()
        .map(normaliser)
        .filter(|m| !m.is_empty())
        .collect();
    if mots.is_empty() {
        return true;
    }
    let botte: Vec<String> = champs.iter().map(|c| normaliser(c)).collect();
    mots.iter()
        .all(|mot| botte.iter().any(|champ| champ.contains(mot.as_str())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_normalisation_efface_accents_et_casse() {
        assert_eq!(normaliser("Λtría - Stretching"), "atria - stretching");
        assert_eq!(normaliser("ÉLITE"), "elite");
        assert_eq!(normaliser("Cœur"), "coeur");
        assert_eq!(normaliser("Straße"), "strasse");
        assert_eq!(normaliser(""), "");
    }

    #[test]
    fn la_normalisation_laisse_passer_ce_qu_elle_ne_connait_pas() {
        // Un emoji ou un ideogramme ne doit pas disparaitre : il fait
        // partie du nom, et quelqu'un peut vouloir le chercher.
        assert_eq!(normaliser("Yox々"), "yox々");
        assert_eq!(normaliser("🦊 Renard"), "🦊 renard");
    }

    #[test]
    fn une_requete_vide_ne_filtre_rien() {
        assert!(correspond(&["n'importe quoi"], ""));
        assert!(correspond(&["n'importe quoi"], "   "));
        assert!(correspond(&[], ""));
    }

    #[test]
    fn chaque_mot_doit_se_trouver_quelque_part() {
        let champs = ["Λtría - Obstacle", "Atria_Obstacle_1"];
        assert!(correspond(&champs, "atria"));
        assert!(correspond(&champs, "obstacle"));
        assert!(correspond(&champs, "atria obstacle"));
        // L'ordre des mots est sans importance.
        assert!(correspond(&champs, "obstacle atria"));
        // Un mot absent suffit a ecarter.
        assert!(!correspond(&champs, "atria elite"));
    }

    #[test]
    fn les_mots_peuvent_venir_de_champs_differents() {
        // Le nom porte l'un, le login porte l'autre.
        assert!(correspond(&["Nac's Bedroom", "nac_bedroom_2"], "bedroom 2"));
    }

    #[test]
    fn la_recherche_ignore_les_accents_dans_la_requete() {
        assert!(correspond(&["Atria"], "Λtría"));
        assert!(correspond(&["Élite Practice"], "elite"));
    }

    #[test]
    fn un_dollar_tape_par_erreur_ne_casse_rien() {
        // Le nom recu est deja nettoye de ses codes, donc un $ dans la
        // requete ne correspond simplement a rien. Il ne doit surtout pas
        // faire disparaitre toute la liste par un effet de bord.
        assert!(!correspond(&["Nac's Bedroom"], "$"));
        assert!(correspond(&["Nac's Bedroom"], "bedroom"));
    }
}
