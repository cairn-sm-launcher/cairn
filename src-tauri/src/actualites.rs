//! Le fil d'actualites : le telechargement, la signature, le cache.
//!
//! Ce module ne decide de rien. Il va chercher deux fichiers, demande a
//! `cairn_core::actualites` s'il faut les croire, et range ce qui a ete
//! cru. Le format, la signature, le markdown et le tri sont la-bas, ou
//! ils sont testes.
//!
//! ## Deux fichiers et pas un
//!
//! Le fil est servi en deux morceaux, `actualites.json` et sa signature
//! `actualites.json.sig`. C'est ce qui permet de **signer les octets tels
//! qu'ils sont servis** au lieu d'un JSON relu puis reecrit. Mettre la
//! signature dans le document qu'elle signe obligerait a la retirer, a
//! reserialiser le reste et a esperer que l'ordre des cles et les espaces
//! tombent pareil des deux cotes. C'est le piege classique, et il se
//! paye en trous de securite plutot qu'en bogues visibles.
//!
//! ## Ce qui est garde sur le disque
//!
//! Les octets du fil et sa signature, tels quels, a cote des reglages.
//! Au demarrage suivant ils sont **reverifies** avant d'etre crus : un
//! cache est un fichier comme un autre sur une machine, et le relire
//! sans controle reviendrait a signer puis a ouvrir la porte derriere.

use cairn_core::actualites::Souci;
use std::time::Duration;

/// Au dela, on rend la main plutot que de laisser l'ecran attendre.
const DELAI: Duration = Duration::from_secs(15);

/// Un fil plus gros que ca n'est pas un fil d'actualites.
///
/// Sans cette borne, un hebergement pris pourrait servir un fichier sans
/// fin et faire gonfler la memoire de Cairn jusqu'a la panne, sans avoir
/// besoin de connaitre la cle de signature.
const TAILLE_MAXIMALE: usize = 2 * 1024 * 1024;

/// Ce que le cache garde, et ce que l'interface recoit.
pub struct Charge {
    pub octets: Vec<u8>,
    pub signature: String,
}

fn dossier() -> Option<std::path::PathBuf> {
    Some(dirs::data_dir()?.join("Cairn"))
}

fn chemin_fil() -> Option<std::path::PathBuf> {
    Some(dossier()?.join("actualites.json"))
}

fn chemin_signature() -> Option<std::path::PathBuf> {
    Some(dossier()?.join("actualites.json.sig"))
}

/// Lit le fil garde sur le disque, **sans encore le croire**.
pub fn depuis_le_cache() -> Option<Charge> {
    let octets = std::fs::read(chemin_fil()?).ok()?;
    let signature = std::fs::read_to_string(chemin_signature()?).ok()?;
    Some(Charge {
        octets,
        signature: signature.trim().to_string(),
    })
}

/// Garde un fil qui vient d'etre verifie.
fn poser_dans_le_cache(charge: &Charge) {
    let (Some(dossier), Some(fil), Some(sig)) = (dossier(), chemin_fil(), chemin_signature())
    else {
        return;
    };
    if std::fs::create_dir_all(&dossier).is_err() {
        return;
    }
    if std::fs::write(&fil, &charge.octets).is_err() {
        crate::journal::alerte("Le fil d'actualites n'a pas pu etre garde en cache.");
        return;
    }
    let _ = std::fs::write(&sig, &charge.signature);
}

/// Va chercher le fil et sa signature.
///
/// Rend une erreur en francais et sans jargon reseau : elle remonte
/// jusqu'a l'ecran, ou personne n'a envie de lire un code HTTP.
pub async fn telecharger() -> Result<Charge, Souci> {
    // Tant qu'aucune adresse n'est renseignee, on ne tente meme pas le
    // reseau : un echec de connexion vers un emplacement reserve n'est
    // pas une information, c'est du bruit dans le journal et un faux
    // message de panne a l'ecran.
    if !cairn_core::actualites_configurees() {
        return Err(Souci::NonConfigure);
    }
    let client = reqwest::Client::builder()
        .timeout(DELAI)
        .user_agent(concat!("Cairn/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| {
            crate::journal::erreur(&format!("Client HTTP impossible a construire : {e}"));
            Souci::Reseau
        })?;

    let octets = recuperer(&client, cairn_core::URL_ACTUALITES).await?;
    if octets.len() > TAILLE_MAXIMALE {
        crate::journal::erreur(&format!(
            "Fil d'actualites refuse : {} octets, au dela de la borne.",
            octets.len()
        ));
        return Err(Souci::TropGros);
    }
    let signature = recuperer(&client, cairn_core::URL_ACTUALITES_SIGNATURE).await?;
    let signature = String::from_utf8_lossy(&signature).trim().to_string();

    Ok(Charge { octets, signature })
}

async fn recuperer(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, Souci> {
    let reponse = client.get(url).send().await.map_err(|e| {
        crate::journal::erreur(&format!("Actualites injoignables : {e}"));
        Souci::Reseau
    })?;
    let code = reponse.status();
    if !code.is_success() {
        crate::journal::erreur(&format!("Actualites : reponse {code}"));
        return Err(Souci::Reseau);
    }
    reponse.bytes().await.map(|b| b.to_vec()).map_err(|e| {
        crate::journal::erreur(&format!("Lecture des actualites interrompue : {e}"));
        Souci::Reseau
    })
}

/// Verifie une charge, puis la lit.
///
/// **La signature d'abord, l'analyse ensuite.** Dans l'autre sens on
/// aurait deja fait tourner l'analyseur sur des octets dont rien ne dit
/// d'ou ils viennent.
pub fn verifier_et_lire(
    charge: &Charge,
    publie_en_cache: Option<i64>,
) -> Result<cairn_core::actualites::Fil, Souci> {
    if !cairn_core::actualites::signature_valable(
        &charge.octets,
        &charge.signature,
        cairn_core::CLE_ACTUALITES,
    ) {
        crate::journal::erreur("Fil d'actualites refuse : signature invalide.");
        return Err(Souci::Signature);
    }
    let fil: cairn_core::actualites::Fil = serde_json::from_slice(&charge.octets).map_err(|e| {
        crate::journal::erreur(&format!("Fil d'actualites illisible : {e}"));
        Souci::Illisible
    })?;
    if !cairn_core::actualites::plus_recent_que(fil.publie, publie_en_cache) {
        crate::journal::alerte(
            "Fil d'actualites refuse : anterieur a celui deja garde, rejeu possible.",
        );
        return Err(Souci::Rejeu);
    }
    Ok(fil)
}

/// Le chemin complet : cache d'abord, reseau ensuite.
///
/// **Le cache est servi avant toute chose**, et il est reverifie en le
/// relisant. Un launcher qui n'affiche rien parce que le reseau est lent
/// est un launcher qui n'affiche rien la moitie du temps.
pub async fn fil(forcer: bool) -> (Option<cairn_core::actualites::Fil>, Option<Souci>) {
    let garde = depuis_le_cache().and_then(|c| verifier_et_lire(&c, None).ok());
    let publie = garde.as_ref().map(|f| f.publie);

    if !forcer && garde.is_some() {
        // On rend ce qu'on a tout de suite ; l'interface redemandera avec
        // `forcer` si elle veut du neuf.
        return (garde, None);
    }

    match telecharger().await {
        Ok(charge) => match verifier_et_lire(&charge, publie) {
            Ok(frais) => {
                poser_dans_le_cache(&charge);
                crate::journal::ecrire(&format!(
                    "Actualites : {} article(s), fil publie le {}.",
                    frais.articles.len(),
                    frais.publie
                ));
                (Some(frais), None)
            }
            // Un fil frais refuse ne remplace jamais un cache valable.
            Err(e) => (garde, Some(e)),
        },
        Err(e) => (garde, Some(e)),
    }
}

/// Ouvre un lien dans le navigateur du systeme.
///
/// **La validation est en Rust et pas dans l'interface**, et elle est
/// refaite ici meme si `cairn-core` a deja ecarte les liens interdits a
/// l'analyse du markdown : l'interface pourrait un jour envoyer autre
/// chose, et ce chemin finit dans un appel systeme qui sait aussi ouvrir
/// un fichier ou un programme.
pub fn ouvrir_lien(lien: &str) -> Result<(), String> {
    if !cairn_core::actualites::lien_autorise(lien) {
        crate::journal::erreur(&format!("Lien refuse : {lien}"));
        return Err("Cairn n'ouvre que des adresses web.".to_string());
    }
    ouvrir(lien)
}

#[cfg(windows)]
fn ouvrir(lien: &str) -> Result<(), String> {
    // Sans shell : l'adresse est un argument et non un morceau de ligne
    // de commande, donc il n'y a rien a y injecter.
    std::process::Command::new("explorer")
        .arg(lien)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            crate::journal::erreur(&format!("Ouverture du lien : {e}"));
            "Cairn n'a pas pu ouvrir ce lien.".to_string()
        })
}

#[cfg(not(windows))]
fn ouvrir(_lien: &str) -> Result<(), String> {
    Err("L'ouverture d'un lien n'existe que sous Windows.".to_string())
}
