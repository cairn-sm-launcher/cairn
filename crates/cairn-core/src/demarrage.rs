//! Le lancement de Cairn au demarrage de Windows.
//!
//! **Ce qu'on ecrit, et ou.** Une seule valeur sous
//! `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`,
//! qui porte le chemin de l'executable. C'est l'endroit prevu pour ca,
//! il ne demande aucune elevation, il ne concerne que l'utilisateur
//! courant, et il se defait depuis le gestionnaire des taches de Windows
//! comme depuis nos reglages. Pas de tache planifiee, pas de service,
//! pas de raccourci pose dans un dossier de demarrage : trois facons
//! plus lourdes de faire la meme chose, et trois facons de laisser
//! quelque chose derriere soi le jour ou l'application est desinstallee.
//!
//! **Et on demarre reduit.** Un launcher qui ouvre sa fenetre en grand
//! par dessus ce qu'on faisait, a chaque ouverture de session, se fait
//! desactiver la premiere semaine. La commande porte donc un drapeau que
//! l'application lit au lancement.
//!
//! Rien ici ne touche au registre : ce module fabrique et relit la
//! chaine, `cairn-app` fait l'ecriture. C'est ce qui permet de tester la
//! partie qui peut se tromper.

/// Le nom de la valeur, donc ce que Windows affiche dans l'onglet
/// Demarrage du gestionnaire des taches.
pub const NOM_VALEUR: &str = "Cairn";

/// La cle, relative a `HKEY_CURRENT_USER`.
pub const CLE: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Le drapeau qui dit a Cairn de s'ouvrir reduit.
pub const DRAPEAU_REDUIT: &str = "--reduit";

/// La commande a inscrire, pour un executable donne.
///
/// Le chemin est entre guillemets, toujours, y compris quand il n'a pas
/// d'espace. Sans eux, `C:\Program Files (x86)\...` serait lu comme la
/// commande `C:\Program` suivie de trois arguments, et Windows ne
/// lancerait rien.
pub fn commande(chemin_exe: &str) -> String {
    format!("\"{}\" {}", chemin_exe.trim(), DRAPEAU_REDUIT)
}

/// Le chemin que porte une valeur deja inscrite, s'il y en a un.
///
/// Accepte les deux formes, avec et sans guillemets : la notre en met,
/// mais une version precedente ou une main humaine a pu en poser une
/// autre, et on doit savoir la reconnaitre pour ne pas en empiler une
/// deuxieme.
pub fn chemin_de(valeur: &str) -> Option<String> {
    let valeur = valeur.trim();
    if let Some(reste) = valeur.strip_prefix('"') {
        let fin = reste.find('"')?;
        return Some(reste[..fin].to_string());
    }
    // Sans guillemets, on s'arrete au premier drapeau et pas au premier
    // espace : un chemin non protege peut en contenir, et le couper la
    // rendrait un chemin faux plutot qu'une absence de chemin.
    let sans_drapeau = match valeur.find(" -") {
        Some(i) => &valeur[..i],
        None => valeur,
    };
    let sans_drapeau = sans_drapeau.trim();
    (!sans_drapeau.is_empty()).then(|| sans_drapeau.to_string())
}

/// Vrai quand la valeur inscrite designe bien cet executable.
///
/// La comparaison ignore la casse : les chemins Windows ne la
/// distinguent pas, et un `C:\` venu du registre peut tres bien
/// s'ecrire `c:\` alors qu'il designe le meme fichier.
pub fn designe(valeur: &str, chemin_exe: &str) -> bool {
    chemin_de(valeur).is_some_and(|c| c.eq_ignore_ascii_case(chemin_exe.trim()))
}

/// Vrai quand les arguments de lancement demandent une fenetre reduite.
pub fn demarre_reduit<I, S>(arguments: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    arguments
        .into_iter()
        .any(|a| a.as_ref().eq_ignore_ascii_case(DRAPEAU_REDUIT))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = r"C:\Program Files (x86)\Cairn\cairn.exe";

    #[test]
    fn la_commande_protege_toujours_le_chemin() {
        let c = commande(EXE);
        assert_eq!(c, "\"C:\\Program Files (x86)\\Cairn\\cairn.exe\" --reduit");
        // Meme sans espace dans le chemin : une regle qui a des
        // exceptions est une regle qu'on oublie d'appliquer.
        assert!(commande(r"C:\Cairn\cairn.exe").starts_with('"'));
    }

    #[test]
    fn on_relit_le_chemin_qu_on_a_ecrit() {
        assert_eq!(chemin_de(&commande(EXE)).unwrap(), EXE);
        assert!(designe(&commande(EXE), EXE));
    }

    #[test]
    fn une_valeur_sans_guillemets_se_lit_aussi() {
        assert_eq!(
            chemin_de(r"C:\Cairn\cairn.exe --reduit").unwrap(),
            r"C:\Cairn\cairn.exe"
        );
        // Et un chemin a espaces non protege est rendu entier plutot que
        // coupe au premier espace.
        assert_eq!(chemin_de(&format!("{EXE} --reduit")).unwrap(), EXE);
        assert_eq!(chemin_de(EXE).unwrap(), EXE);
    }

    #[test]
    fn la_casse_du_chemin_ne_compte_pas() {
        assert!(designe(
            "\"c:\\program files (x86)\\cairn\\CAIRN.EXE\" --reduit",
            EXE
        ));
    }

    #[test]
    fn une_valeur_qui_designe_autre_chose_est_reconnue_comme_telle() {
        assert!(!designe(r#""C:\Autre\truc.exe" --reduit"#, EXE));
        assert!(!designe("", EXE));
        assert!(!designe("   ", EXE));
        // Des guillemets jamais refermes ne donnent pas de chemin.
        assert_eq!(chemin_de("\"C:\\Cairn\\cairn.exe"), None);
    }

    #[test]
    fn le_drapeau_se_reconnait_parmi_les_arguments() {
        assert!(demarre_reduit(["cairn.exe", "--reduit"]));
        assert!(demarre_reduit(["cairn.exe", "--REDUIT"]));
        assert!(!demarre_reduit(["cairn.exe"]));
        assert!(!demarre_reduit(["cairn.exe", "--reduitmoi"]));
        let vide: [&str; 0] = [];
        assert!(!demarre_reduit(vide));
    }
}
