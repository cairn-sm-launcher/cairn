//! Ce que Cairn range sur le disque : le fichier de reglages, et rien
//! d'autre pour l'instant.
//!
//! `%APPDATA%\Cairn\reglages.json`. La forme, les valeurs par defaut et le
//! controle de ce qui remonte sont dans `cairn_core::reglages`, qui les
//! teste. Ici il n'y a que le disque.
//!
//! Deux partis pris.
//!
//! **Rien ne fait echouer le demarrage.** Fichier absent, illisible, JSON
//! casse : Cairn ouvre sur les reglages par defaut et le note au journal.
//! Un launcher qui refuse de s'ouvrir parce qu'un fichier de preferences
//! est abime serait un mauvais launcher.
//!
//! **L'ecriture est atomique.** On ecrit a cote puis on renomme. Sans ca,
//! une coupure de courant au mauvais moment laisse un fichier tronque, et
//! le joueur retrouve ses reglages par defaut au lancement suivant.

use cairn_core::reglages::Reglages;
use std::path::PathBuf;

/// `%APPDATA%\Cairn\reglages.json`, ou l'equivalent local ailleurs.
pub fn chemin() -> Option<PathBuf> {
    Some(dirs::data_dir()?.join("Cairn").join("reglages.json"))
}

/// Lit les reglages, ou rend ceux d'un premier lancement.
pub fn lire() -> Reglages {
    let Some(chemin) = chemin() else {
        crate::journal::alerte("Aucun dossier de donnees, reglages par defaut.");
        return premier_lancement();
    };

    let texte = match std::fs::read_to_string(&chemin) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            crate::journal::ecrire("Premier lancement : aucun fichier de reglages.");
            return premier_lancement();
        }
        Err(e) => {
            crate::journal::alerte(&format!("Reglages illisibles ({e}), valeurs par defaut."));
            return Reglages::default();
        }
    };

    match serde_json::from_str::<Reglages>(&texte) {
        Ok(r) => r.assainir(),
        Err(e) => {
            // Le fichier est mis de cote plutot qu'ecrase : si un joueur l'a
            // bricole, il peut encore aller voir ce qu'il avait ecrit.
            let ecarte = chemin.with_extension("json.invalide");
            let _ = std::fs::rename(&chemin, &ecarte);
            crate::journal::alerte(&format!(
                "Reglages illisibles ({e}). Le fichier est mis de cote sous {}, \
                 Cairn repart sur les valeurs par defaut.",
                ecarte.display()
            ));
            Reglages::default()
        }
    }
}

fn premier_lancement() -> Reglages {
    let locale = sys_locale::get_locale().unwrap_or_default();
    let r = Reglages::premier_lancement(&locale);
    crate::journal::ecrire(&format!(
        "Langue du systeme : {}, Cairn demarre en {}.",
        if locale.is_empty() {
            "inconnue"
        } else {
            &locale
        },
        r.langue
    ));
    r
}

/// Enregistre les reglages. Rend un message lisible en cas d'echec.
pub fn ecrire(reglages: &Reglages) -> Result<(), String> {
    let chemin = chemin().ok_or("Cairn n'a pas trouve ou ranger ses reglages.")?;
    let dossier = chemin
        .parent()
        .ok_or("Cairn n'a pas trouve ou ranger ses reglages.")?;
    std::fs::create_dir_all(dossier).map_err(|e| {
        crate::journal::erreur(&format!("Dossier de reglages ({e})."));
        "Cairn n'a pas pu creer son dossier de reglages.".to_string()
    })?;

    let texte = serde_json::to_string_pretty(reglages)
        .map_err(|e| format!("Reglages non serialisables : {e}"))?;

    // A cote, puis renommage. Le renommage est atomique sur le meme volume,
    // donc le fichier vu par le prochain lancement est soit l'ancien entier,
    // soit le nouveau entier, jamais un melange.
    let provisoire = chemin.with_extension("json.nouveau");
    std::fs::write(&provisoire, texte).map_err(|e| {
        crate::journal::erreur(&format!("Ecriture des reglages ({e})."));
        "Cairn n'a pas pu enregistrer les reglages.".to_string()
    })?;
    std::fs::rename(&provisoire, &chemin).map_err(|e| {
        let _ = std::fs::remove_file(&provisoire);
        crate::journal::erreur(&format!("Remplacement des reglages ({e})."));
        "Cairn n'a pas pu enregistrer les reglages.".to_string()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_chemin_va_bien_dans_un_dossier_cairn() {
        if let Some(c) = chemin() {
            assert!(c.ends_with("Cairn/reglages.json") || c.ends_with(r"Cairn\reglages.json"));
        }
    }

    #[test]
    fn un_aller_retour_sur_disque_rend_les_memes_reglages() {
        // On ecrit a cote du dossier reel pour ne pas toucher aux reglages
        // d'un vrai poste pendant les tests.
        let dossier = std::env::temp_dir().join(format!("cairn-test-{}", std::process::id()));
        std::fs::create_dir_all(&dossier).unwrap();
        let fichier = dossier.join("reglages.json");

        let avant = Reglages {
            langue: "fr".to_string(),
            accent: "#A06BFF".to_string(),
            assistant_fait: true,
            ..Reglages::default()
        };
        std::fs::write(&fichier, serde_json::to_string_pretty(&avant).unwrap()).unwrap();

        let texte = std::fs::read_to_string(&fichier).unwrap();
        let apres: Reglages = serde_json::from_str(&texte).unwrap();
        assert_eq!(avant, apres.assainir());

        let _ = std::fs::remove_dir_all(&dossier);
    }
}
