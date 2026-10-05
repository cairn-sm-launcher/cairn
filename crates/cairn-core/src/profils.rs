//! Les profils locaux du jeu.
//!
//! Un profil est un fichier `<nom>.Profile.Gbx` dans le dossier de
//! configuration de ManiaPlanet. Aucun mot de passe n'entre en jeu : le
//! launcher dit seulement au jeu lequel demarrer, le jeu fait le reste.
//!
//! Le parcours du dossier est une entree-sortie et vit dans `cairn-app`.
//! Ce qui est ici, c'est le nom qu'on en tire et l'ordre dans lequel on les
//! presente.

use serde::Serialize;

/// Un profil tel que l'interface le recoit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Profil {
    /// Identifiant, egal au nom de fichier sans son extension.
    pub id: String,
    /// Ce qui s'affiche.
    pub nom: String,
    /// Date de derniere utilisation, au format AAAA-MM-JJ, telle que
    /// l'appelant l'a lue sur le fichier. `None` si on ne sait pas.
    ///
    /// C'est une date et pas une phrase, volontairement : un coeur metier
    /// qui rendrait « Last used 2026-09-24 » imposerait sa langue a
    /// l'interface, qui n'aurait plus qu'a la traduire par motif.
    pub derniere_utilisation: Option<String>,
    /// La couleur de la pastille, voir [`teinte`].
    pub teinte: String,
    /// La lettre affichee dans la pastille, voir [`initiale`].
    pub initiale: String,
}

/// Les six teintes de pastille, reprises des couleurs d'accent.
const TEINTES: [&str; 6] = [
    "#20E3B2", "#5B8CFF", "#FFC857", "#A06BFF", "#FF6B4A", "#8FA0B5",
];

/// La couleur de pastille d'un profil.
///
/// Elle est tiree du nom et pas du rang dans la liste. La liste est
/// triee par date de derniere utilisation, donc elle se reordonne
/// toute seule : avec un choix par rang, jouer sur un profil ferait
/// changer la couleur de tous les autres. Tiree du nom, elle ne bouge
/// jamais, et c'est elle qui fait reconnaitre un profil du coin de
/// l'oeil avant d'en avoir lu le nom.
pub fn teinte(nom: &str) -> &'static str {
    // Somme ponderee sur les octets. Ce n'est pas une empreinte
    // cryptographique et ca n'a pas a l'etre : il s'agit de repartir six
    // couleurs, pas de resister a quoi que ce soit.
    let empreinte = nom
        .bytes()
        .fold(0u32, |acc, o| acc.wrapping_mul(31).wrapping_add(o as u32));
    TEINTES[(empreinte % TEINTES.len() as u32) as usize]
}

/// La lettre affichee dans la pastille.
///
/// Le premier caractere du nom, en majuscule. Un nom qui commence par un
/// emoji ou par une lettre non latine rend ce caractere tel quel, ce qui
/// vaut mieux qu'un point d'interrogation. Un nom vide rend `?`, mais
/// [`nom_depuis_fichier`] a deja ecarte ce cas.
pub fn initiale(nom: &str) -> String {
    match nom.trim().chars().next() {
        Some(c) => c.to_uppercase().collect(),
        None => "?".to_string(),
    }
}

/// Tire le nom d'un profil de son nom de fichier.
///
/// Rend `None` si le fichier n'est pas un profil, ce qui evite de faire
/// passer pour un profil un `Cache.Gbx` ou un `.Profile.Gbx.bak` qui traine
/// dans le meme dossier.
pub fn nom_depuis_fichier(nom_fichier: &str) -> Option<String> {
    let nom = nom_fichier.strip_suffix(".Profile.Gbx")?;
    let nom = nom.trim();
    (!nom.is_empty()).then(|| nom.to_string())
}

/// Construit un profil a partir de son fichier et d'une date deja mise en
/// forme par l'appelant, qui est le seul a savoir lire une date de fichier.
pub fn depuis_fichier(nom_fichier: &str, derniere_utilisation: Option<&str>) -> Option<Profil> {
    let nom = nom_depuis_fichier(nom_fichier)?;
    Some(Profil {
        id: nom.clone(),
        derniere_utilisation: derniere_utilisation.map(str::to_string),
        teinte: teinte(&nom).to_string(),
        initiale: initiale(&nom),
        nom,
    })
}

/// Choisit le profil actif parmi ceux qui existent vraiment.
///
/// Si le profil memorise n'existe plus, parce qu'il a ete renomme ou
/// supprime dans le jeu, on retombe sur le premier de la liste. C'est le
/// point ou la passerelle du lot 18 se trompait en silence : elle gardait un
/// identifiant fantome et le lancement echouait sans explication.
pub fn profil_actif(profils: &[Profil], memorise: Option<&str>) -> Option<String> {
    if let Some(id) = memorise {
        if profils.iter().any(|p| p.id == id) {
            return Some(id.to_string());
        }
    }
    profils.first().map(|p| p.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profil(id: &str) -> Profil {
        Profil {
            id: id.to_string(),
            nom: id.to_string(),
            derniere_utilisation: None,
            teinte: teinte(id).to_string(),
            initiale: initiale(id),
        }
    }

    #[test]
    fn le_nom_est_le_fichier_sans_son_extension() {
        assert_eq!(
            nom_depuis_fichier("Xirdrak.Profile.Gbx").unwrap(),
            "Xirdrak"
        );
        assert_eq!(
            nom_depuis_fichier("Mon Profil 2.Profile.Gbx").unwrap(),
            "Mon Profil 2"
        );
    }

    #[test]
    fn ce_qui_n_est_pas_un_profil_est_ecarte() {
        assert_eq!(nom_depuis_fichier("Cache.Gbx"), None);
        assert_eq!(nom_depuis_fichier("Xirdrak.Profile.Gbx.bak"), None);
        assert_eq!(nom_depuis_fichier(".Profile.Gbx"), None);
        assert_eq!(nom_depuis_fichier(""), None);
    }

    #[test]
    fn la_date_est_rendue_telle_quelle_sans_phrase_autour() {
        let p = depuis_fichier("Xirdrak.Profile.Gbx", Some("2026-09-24")).unwrap();
        assert_eq!(p.id, "Xirdrak");
        assert_eq!(p.nom, "Xirdrak");
        assert_eq!(p.derniere_utilisation.as_deref(), Some("2026-09-24"));

        let sans = depuis_fichier("Xirdrak.Profile.Gbx", None).unwrap();
        assert_eq!(sans.derniere_utilisation, None);
    }

    #[test]
    fn la_teinte_suit_le_nom_et_pas_le_rang() {
        // Le meme nom donne toujours la meme teinte.
        assert_eq!(teinte("Xirdrak"), teinte("Xirdrak"));
        // Et elle fait toujours partie des six.
        for nom in ["Xirdrak", "Guest", "", "Mon Profil 2", "Émile"] {
            assert!(TEINTES.contains(&teinte(nom)), "{nom}");
        }
        // Deux noms voisins ne tombent pas systematiquement sur la meme.
        let distinctes: std::collections::HashSet<_> = ["A", "B", "C", "D", "E", "F"]
            .iter()
            .map(|n| teinte(n))
            .collect();
        assert!(distinctes.len() >= 5, "{distinctes:?}");
    }

    #[test]
    fn l_initiale_est_la_premiere_lettre_en_majuscule() {
        assert_eq!(initiale("xirdrak"), "X");
        assert_eq!(initiale("  guest"), "G");
        assert_eq!(initiale("Émile"), "É");
        assert_eq!(initiale("9lives"), "9");
        assert_eq!(initiale(""), "?");
        // Un nom qui commence par un emoji rend l'emoji, ce qui vaut
        // mieux qu'un caractere de remplacement.
        assert_eq!(initiale("🦊 Renard"), "🦊");
    }

    #[test]
    fn un_profil_memorise_qui_existe_encore_est_conserve() {
        let liste = [profil("A"), profil("B")];
        assert_eq!(profil_actif(&liste, Some("B")).unwrap(), "B");
    }

    #[test]
    fn un_profil_memorise_disparu_retombe_sur_le_premier() {
        let liste = [profil("A"), profil("B")];
        assert_eq!(profil_actif(&liste, Some("Fantome")).unwrap(), "A");
        assert_eq!(profil_actif(&liste, None).unwrap(), "A");
    }

    #[test]
    fn sans_aucun_profil_il_n_y_a_pas_de_profil_actif() {
        assert_eq!(profil_actif(&[], Some("A")), None);
        assert_eq!(profil_actif(&[], None), None);
    }
}
