//! L'inscription de Cairn au demarrage de Windows.
//!
//! Tout ce qui peut se tromper est dans `cairn_core::demarrage`, ou
//! c'est teste : la forme de la commande, la relecture du chemin, la
//! comparaison insensible a la casse. Ce qui reste ici est l'acces au
//! registre, trois appels, et il est volontairement aussi court que ca.
//!
//! **Une seule valeur, sous la ruche de l'utilisateur courant.** Pas
//! d'elevation, rien qui touche aux autres comptes, et Windows la montre
//! dans l'onglet Demarrage de son gestionnaire des taches, ou elle se
//! decoche sans nous. C'est pour ca que l'etat reel est relu au
//! lancement plutot que cru sur parole, voir `run`.

use cairn_core::demarrage;

/// Le chemin de l'executable en cours.
///
/// Sous Windows seulement : ailleurs, rien ne l'utilise, et une
/// fonction morte est un avertissement de plus a la compilation.
#[cfg(windows)]
fn chemin_executable() -> Option<String> {
    std::env::current_exe()
        .ok()
        .map(|c| c.display().to_string())
}

/// Cairn est-il inscrit au demarrage, pour CET executable.
///
/// « Pour cet executable » compte : une inscription qui designe une
/// copie deplacee ou desinstallee ne lancerait rien, et la montrer
/// comme active serait mentir.
#[cfg(windows)]
pub fn actif() -> bool {
    let Some(exe) = chemin_executable() else {
        return false;
    };
    let racine = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let Ok(cle) = racine.open_subkey(demarrage::CLE) else {
        return false;
    };
    let valeur: Result<String, _> = cle.get_value(demarrage::NOM_VALEUR);
    valeur.is_ok_and(|v| demarrage::designe(&v, &exe))
}

#[cfg(not(windows))]
pub fn actif() -> bool {
    false
}

/// Inscrit ou retire Cairn du demarrage.
///
/// Retirer une valeur qui n'existe pas n'est pas une erreur : c'est
/// l'etat demande, et il est atteint.
#[cfg(windows)]
pub fn appliquer(souhaite: bool) -> Result<(), String> {
    let racine = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if !souhaite {
        let Ok(cle) = racine.open_subkey_with_flags(demarrage::CLE, winreg::enums::KEY_WRITE)
        else {
            return Ok(());
        };
        match cle.delete_value(demarrage::NOM_VALEUR) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => {
                crate::journal::erreur(&format!("Retrait du demarrage impossible : {e}"));
                Err("Cairn n'a pas pu se retirer du demarrage de Windows.".to_string())
            }
        }
    } else {
        let Some(exe) = chemin_executable() else {
            return Err("Cairn n'a pas trouve son propre executable.".to_string());
        };
        let (cle, _) = racine.create_subkey(demarrage::CLE).map_err(|e| {
            crate::journal::erreur(&format!("Cle Run inaccessible : {e}"));
            "Cairn n'a pas pu ecrire dans le registre.".to_string()
        })?;
        cle.set_value(demarrage::NOM_VALEUR, &demarrage::commande(&exe))
            .map_err(|e| {
                crate::journal::erreur(&format!("Inscription au demarrage impossible : {e}"));
                "Cairn n'a pas pu s'inscrire au demarrage de Windows.".to_string()
            })?;
        crate::journal::ecrire("Inscrit au demarrage de Windows.");
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn appliquer(_souhaite: bool) -> Result<(), String> {
    Ok(())
}

/// La fenetre doit-elle s'ouvrir reduite.
///
/// Vrai quand Windows nous a lances lui meme au demarrage de la session,
/// puisque c'est nous qui avons ecrit ce drapeau dans la commande.
pub fn ouvrir_reduit() -> bool {
    demarrage::demarre_reduit(std::env::args())
}
