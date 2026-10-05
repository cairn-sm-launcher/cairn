//! Demarrer le jeu.
//!
//! `cairn-core` a deja decide de la ligne de commande et verifie que rien
//! d'hostile ne s'y trouve. Ce module ne fait que l'executer, et il a une
//! seule subtilite, qui vaut tous les commentaires du monde : la ligne est
//! passee **brute**.

use cairn_core::lancement::{self, Action, Lancement};
use serde::Serialize;

/// Ce que l'interface apprend d'une tentative de lancement.
#[derive(Debug, Clone, Serialize)]
pub struct Rejointe {
    /// `true` quand le jeu tournait deja et qu'on lui a passe la main.
    pub relais: bool,
}

/// Ouvre le jeu sur ses propres menus.
///
/// Sert au bouton « Ouvrir les reglages du jeu » : les reglages
/// graphiques, audio, reseau et touches vivent dans des fichiers
/// binaires que Cairn ne touche pas, donc la seule facon de les changer
/// est d'aller dans le jeu.
pub fn ouvrir(exe: &str, profil: &str, jeu_lance: bool) -> Result<Rejointe, String> {
    if jeu_lance {
        // Relancer ferait apparaitre un second processus qui passerait la
        // main au premier, sans rien ouvrir de visible. Mieux vaut le
        // dire que de ne rien faire en silence.
        return Err("ManiaPlanet tourne deja, les reglages sont dans sa fenetre.".to_string());
    }
    if !std::path::Path::new(exe).is_file() {
        crate::journal::erreur(&format!("{exe} n'existe pas."));
        return Err("ManiaPlanet.exe est introuvable.".to_string());
    }

    let prepare = lancement::ouvrir(profil).map_err(|refus| {
        crate::journal::erreur(&format!("Ouverture refusee : {refus}"));
        "Ce profil porte un nom que Cairn refuse d'executer.".to_string()
    })?;

    crate::journal::ecrire(&format!(
        "Ouverture des menus : \"{exe}\" {}",
        prepare.ligne
    ));
    executer(exe, &prepare)
}

/// Ouvre le jeu sur le menu d'un title pack.
///
/// C'est l'action des tuiles de l'ecran Editer. Contrairement a
/// [`ouvrir`], elle marche **aussi quand le jeu tourne deja** : le
/// protocole fait basculer l'instance existante sur le menu demande, ce
/// qui est exactement le service rendu. C'est pour ca que les deux
/// fonctions restent separees plutot que de partager un drapeau.
pub fn ouvrir_titre(
    exe: &str,
    profil: &str,
    titre: &str,
    jeu_lance: bool,
) -> Result<Rejointe, String> {
    if !std::path::Path::new(exe).is_file() {
        crate::journal::erreur(&format!("{exe} n'existe pas."));
        return Err("ManiaPlanet.exe est introuvable.".to_string());
    }

    let prepare = lancement::ouvrir_titre(profil, titre, jeu_lance).map_err(|refus| {
        crate::journal::erreur(&format!("Ouverture de titre refusee : {refus}"));
        "Ce title pack porte un nom que Cairn refuse d'executer.".to_string()
    })?;

    crate::journal::ecrire(&format!("Ouverture du titre : \"{exe}\" {}", prepare.ligne));
    executer(exe, &prepare)
}

/// Prepare puis execute le lancement.
pub fn rejoindre(
    exe: &str,
    profil: &str,
    login: &str,
    titre: &str,
    spectate: bool,
    jeu_lance: bool,
) -> Result<Rejointe, String> {
    if !std::path::Path::new(exe).is_file() {
        crate::journal::erreur(&format!("{exe} n'existe pas."));
        return Err("ManiaPlanet.exe est introuvable.".to_string());
    }

    let action = Action::depuis_spectate(spectate);
    let prepare =
        lancement::preparer(profil, login, titre, action, jeu_lance).map_err(|refus| {
            // Un refus n'est pas un incident technique, c'est une donnee qui ne
            // passe pas le controle. On le note tel quel, il sera precieux si un
            // serveur du jeu se met a porter un login exotique.
            crate::journal::erreur(&format!("Lancement refuse : {refus}"));
            "Ce serveur porte un identifiant que Cairn refuse d'executer.".to_string()
        })?;

    crate::journal::ecrire(&format!(
        "Lancement ({}) : \"{exe}\" {}",
        if prepare.relais {
            "jeu deja lance"
        } else {
            "jeu ferme"
        },
        prepare.ligne
    ));

    executer(exe, &prepare)
}

#[cfg(windows)]
fn executer(exe: &str, prepare: &Lancement) -> Result<Rejointe, String> {
    // Le point a ne jamais perdre de vue. `Command::arg` applique
    // l'echappement standard de Windows, qui transformerait
    // /url="maniaplanet://..." en "/url=\"maniaplanet://...\"". Le jeu
    // analyse sa ligne de commande a la main : il ignorerait l'argument
    // sans un mot d'explication. `raw_arg` la passe telle quelle.
    use std::os::windows::process::CommandExt;

    let mut commande = std::process::Command::new(exe);
    commande.raw_arg(&prepare.ligne);

    // Le jeu doit demarrer depuis son propre dossier, sinon il ne trouve
    // pas ses donnees quand il est lance depuis ailleurs.
    if let Some(dossier) = std::path::Path::new(exe).parent() {
        commande.current_dir(dossier);
    }

    let mut processus = commande.spawn().map_err(|e| {
        crate::journal::erreur(&format!("Demarrage de ManiaPlanet : {e}"));
        "Cairn n'a pas pu demarrer ManiaPlanet.".to_string()
    })?;

    if prepare.relais {
        // Ce processus la n'est qu'un messager : /singleinst lui fait
        // passer la consigne a l'instance qui tourne deja, puis il s'arrete.
        // Son code de sortie ne dit donc rien de la reussite, et on ne
        // l'attend que pour le noter.
        std::thread::sleep(std::time::Duration::from_millis(1200));
        match processus.try_wait() {
            Ok(Some(code)) => crate::journal::ecrire(&format!(
                "Le messager a quitte ({code}) : la main est passee a l'instance existante."
            )),
            Ok(None) => crate::journal::alerte("Le messager tourne encore apres 1,2 s."),
            Err(e) => crate::journal::alerte(&format!("Etat du messager illisible : {e}")),
        }
    }

    Ok(Rejointe {
        relais: prepare.relais,
    })
}

#[cfg(not(windows))]
fn executer(_exe: &str, _prepare: &Lancement) -> Result<Rejointe, String> {
    // Cairn ne vise que Windows. Ce bras existe pour que la caisse compile
    // et se teste ailleurs, pas pour faire croire a un support.
    Err("Le lancement du jeu n'est disponible que sous Windows.".to_string())
}
