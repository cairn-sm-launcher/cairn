//! Le journal de Cairn, cote ecriture.
//!
//! Un fichier texte dans `%APPDATA%\Cairn\logs\cairn.log`. Il sert a une
//! seule chose : qu'un lancement qui echoue laisse une trace lisible par
//! quelqu'un qui n'est pas developpeur, et transmissible sans crainte.
//!
//! **Ce qui n'entre jamais ici** : un jeton, un mot de passe, un contenu
//! recu du reseau autre que ce qui est necessaire au diagnostic. Le chemin
//! du jeu et le nom de profil y sont, ils sont anodins, mais la regle est
//! posee maintenant pour ne pas avoir a nettoyer un journal plus tard.
//!
//! **Le niveau est ecrit dans la ligne depuis le lot 23.** Il s'ecrivait
//! avant a la main, en commencant certains messages par le mot ERREUR, et
//! seulement quand on y pensait. Un filtre par niveau construit sur cette
//! convention aurait ete un filtre qui passe au vert sur du faux. La
//! lecture, elle, accepte toujours les lignes de l'ancien format, voir
//! `cairn_core::journal` : le fichier deja present sur les machines est
//! ecrit comme ca.

use cairn_core::journal::Niveau;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Au dela de cette taille le journal est archive et reparti a zero. Sans
/// cette limite un launcher qui tourne des mois finit par ecrire un fichier
/// que plus personne n'ouvre.
const TAILLE_MAXIMALE: u64 = 1024 * 1024;

/// `%APPDATA%\Cairn\logs\cairn.log`, ou l'equivalent local ailleurs.
pub fn chemin() -> Option<PathBuf> {
    Some(
        dirs::data_dir()?
            .join("Cairn")
            .join("logs")
            .join("cairn.log"),
    )
}

/// Le cours normal des choses.
pub fn ecrire(message: &str) {
    poser(Niveau::Info, message);
}

/// Quelque chose n'a pas marche, Cairn a continue autrement.
pub fn alerte(message: &str) {
    poser(Niveau::Alerte, message);
}

/// Une action demandee n'a pas eu lieu.
pub fn erreur(message: &str) {
    poser(Niveau::Erreur, message);
}

/// Ecrit une ligne horodatee et classee.
///
/// Ne rend rien et n'echoue jamais visiblement : un journal qui empeche
/// l'application de fonctionner est pire que pas de journal du tout.
fn poser(niveau: Niveau, message: &str) {
    let horodatage = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let etiquette = niveau.etiquette();
    // Un message sur plusieurs lignes garderait un retour a la ligne au
    // milieu d'une entree. La lecture sait le rattacher, mais autant ne
    // pas fabriquer le cas : les retours deviennent des espaces.
    let message = message.replace(['\n', '\r'], " ");
    // En developpement, la console ouverte par `main.rs` recoit la meme
    // chose : on ne va pas chercher un fichier quand on a un terminal.
    println!("{horodatage} | {etiquette} | {message}");

    let Some(chemin) = chemin() else { return };
    let Some(dossier) = chemin.parent() else {
        return;
    };
    if fs::create_dir_all(dossier).is_err() {
        return;
    }
    archiver_si_trop_gros(&chemin);

    if let Ok(mut fichier) = OpenOptions::new().create(true).append(true).open(&chemin) {
        let _ = writeln!(fichier, "{horodatage} | {etiquette} | {message}");
    }
}

/// Le journal tel qu'il est sur le disque, pour le lecteur du lot 23.
///
/// **Aucun chemin ne vient de l'interface**, ici comme pour l'ouverture
/// de dossier du lot 22d : Cairn resout le sien et lui seul. Il n'y a
/// donc rien a valider, parce qu'il n'y a rien a composer.
///
/// L'archive precede le fichier courant quand elle existe, pour que la
/// lecture soit chronologique d'un bout a l'autre.
pub fn lire() -> String {
    let Some(chemin) = chemin() else {
        return String::new();
    };
    let mut texte = String::new();
    if let Ok(archive) = fs::read_to_string(chemin.with_extension("log.1")) {
        texte.push_str(&archive);
        if !texte.ends_with('\n') {
            texte.push('\n');
        }
    }
    if let Ok(courant) = fs::read_to_string(&chemin) {
        texte.push_str(&courant);
    }
    texte
}

fn archiver_si_trop_gros(chemin: &PathBuf) {
    let Ok(metadonnees) = fs::metadata(chemin) else {
        return;
    };
    if metadonnees.len() < TAILLE_MAXIMALE {
        return;
    }
    // Une seule generation d'archive. Deux, c'est deja de la conservation,
    // et ce n'est pas le role d'un launcher.
    let _ = fs::rename(chemin, chemin.with_extension("log.1"));
}
