//! Retrouver l'installation du jeu.
//!
//! La source la plus fiable est le registre : le jeu y inscrit lui meme la
//! commande associee au protocole `maniaplanet://` au moment ou il
//! s'enregistre. Elle suit l'installation ou qu'elle soit, y compris sur une
//! bibliotheque Steam secondaire.
//!
//! La lecture du registre est une entree-sortie et vit donc dans
//! `cairn-app`. Ce qui est ici, c'est l'analyse de la chaine obtenue, qui est
//! pure et qui merite des tests : elle a plusieurs formes.

/// Extrait le chemin de l'executable d'une commande de registre.
///
/// Les deux formes rencontrees :
///
/// ```text
/// "C:\Program Files (x86)\ManiaPlanet\ManiaPlanet.exe" /useexedir /url="%1"
/// C:\Jeux\ManiaPlanet\ManiaPlanet.exe /useexedir
/// ```
///
/// La premiere est la normale, la seconde arrive sur les installations sans
/// espace dans le chemin. Un chemin non entoure de guillemets **et**
/// contenant un espace est indecidable, on ne devine pas.
pub fn exe_depuis_commande(commande: &str) -> Option<String> {
    let commande = commande.trim();
    if commande.is_empty() {
        return None;
    }

    // Forme entre guillemets : tout ce qui est avant le guillemet fermant.
    if let Some(reste) = commande.strip_prefix('"') {
        let fin = reste.find('"')?;
        let chemin = reste[..fin].trim();
        return (!chemin.is_empty()).then(|| chemin.to_string());
    }

    // Forme nue : on s'arrete au premier `.exe`, extension comprise.
    let minuscule = commande.to_ascii_lowercase();
    let fin = minuscule.find(".exe")? + ".exe".len();
    let chemin = commande[..fin].trim();
    (!chemin.is_empty()).then(|| chemin.to_string())
}

/// Les emplacements a essayer quand le registre ne dit rien.
///
/// Repli volontairement court. Une liste longue donne l'illusion de couvrir
/// tous les cas alors qu'elle rate les bibliotheques Steam secondaires, qui
/// sont justement le cas difficile. Le vrai filet, c'est que l'utilisateur
/// peut designer l'exe a la main dans l'assistant de premier lancement.
pub fn chemins_de_repli() -> Vec<&'static str> {
    vec![
        r"C:\Program Files (x86)\ManiaPlanet\ManiaPlanet.exe",
        r"C:\Program Files (x86)\Steam\steamapps\common\ManiaPlanet_SMStorm\ManiaPlanet.exe",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forme_entre_guillemets_avec_arguments() {
        let c = r#""C:\Program Files (x86)\ManiaPlanet\ManiaPlanet.exe" /useexedir /url="%1""#;
        assert_eq!(
            exe_depuis_commande(c).unwrap(),
            r"C:\Program Files (x86)\ManiaPlanet\ManiaPlanet.exe"
        );
    }

    #[test]
    fn forme_nue_sans_espace_dans_le_chemin() {
        let c = r"C:\Jeux\ManiaPlanet\ManiaPlanet.exe /useexedir /singleinst";
        assert_eq!(
            exe_depuis_commande(c).unwrap(),
            r"C:\Jeux\ManiaPlanet\ManiaPlanet.exe"
        );
    }

    #[test]
    fn forme_nue_sans_aucun_argument() {
        let c = r"D:\MP\ManiaPlanet.exe";
        assert_eq!(exe_depuis_commande(c).unwrap(), r"D:\MP\ManiaPlanet.exe");
    }

    #[test]
    fn l_extension_est_reconnue_quelle_que_soit_la_casse() {
        let c = r"C:\MP\ManiaPlanet.EXE /useexedir";
        assert_eq!(exe_depuis_commande(c).unwrap(), r"C:\MP\ManiaPlanet.EXE");
    }

    #[test]
    fn les_espaces_de_bord_du_registre_sont_ignores() {
        let c = "  \"C:\\MP\\ManiaPlanet.exe\" /x  ";
        assert_eq!(exe_depuis_commande(c).unwrap(), r"C:\MP\ManiaPlanet.exe");
    }

    #[test]
    fn ce_qui_n_est_pas_decidable_ne_rend_rien() {
        assert_eq!(exe_depuis_commande(""), None);
        assert_eq!(exe_depuis_commande("   "), None);
        // Guillemet ouvrant jamais referme.
        assert_eq!(exe_depuis_commande(r#""C:\MP\ManiaPlanet.exe /x"#), None);
        // Aucun executable nomme.
        assert_eq!(exe_depuis_commande("rundll32 quelquechose"), None);
        // Guillemets vides.
        assert_eq!(exe_depuis_commande(r#""" /x"#), None);
    }

    #[test]
    fn le_repli_pointe_bien_sur_des_executables() {
        let repli = chemins_de_repli();
        assert!(!repli.is_empty());
        assert!(repli.iter().all(|c| c.ends_with("ManiaPlanet.exe")));
    }
}
