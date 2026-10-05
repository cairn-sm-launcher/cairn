//! L'appel aux WebServices de ManiaPlanet.
//!
//! Seul point de sortie reseau de Cairn, et il est volontairement unique :
//! une seule adresse, en dur dans `cairn-core`, sans rien qui vienne de
//! l'exterieur. La traduction de ce qu'on recoit est dans `cairn-core`, ce
//! module ne fait que l'aller-retour.

use std::time::Duration;

use cairn_core::serveurs::{self, Serveur, ServeurApi};

/// Au dela, on rend la main plutot que de laisser l'interface attendre.
const DELAI: Duration = Duration::from_secs(20);

/// Recupere les serveurs ShootMania en ligne.
///
/// **Pourquoi plusieurs appels.** L'API ne sait pas filtrer sur
/// l'environnement : elle rend tout le flux ManiaPlanet, ou Trackmania
/// est largement majoritaire, et c'est nous qui gardons ShootMania. Une
/// page unique laisserait donc dehors des serveurs Storm sans qu'on
/// puisse savoir lesquels. On parcourt jusqu'a la fin de la liste, ou
/// jusqu'au garde-fou.
///
/// Les messages d'erreur sont en francais et sans jargon reseau : ils
/// remontent jusqu'a l'ecran, ou personne n'a envie de lire un code HTTP.
pub async fn serveurs_en_ligne() -> Result<Vec<Serveur>, String> {
    let client = reqwest::Client::builder()
        .timeout(DELAI)
        .user_agent(concat!("Cairn/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| {
            crate::journal::erreur(&format!("Client HTTP impossible a construire : {e}"));
            "Impossible de preparer la connexion.".to_string()
        })?;

    let taille = cairn_core::PAGE_SERVEURS;
    let mut bruts: Vec<ServeurApi> = Vec::new();
    let mut pages = 0;
    let mut premier_login_precedent: Option<String> = None;

    for page in 0..cairn_core::PAGES_SERVEURS_MAX {
        let url = format!(
            "{}&offset={}&length={}",
            cairn_core::URL_SERVEURS,
            page * taille,
            taille
        );
        let recus = une_page(&client, &url).await?;
        pages += 1;

        if recus.is_empty() {
            break;
        }

        // Le garde-fou. Si l'API ignorait `offset`, chaque page rendrait
        // la premiere, et la boucle tournerait jusqu'au maximum en
        // empilant huit fois les memes serveurs. On s'en apercoit au
        // premier login, qui ne peut pas etre le meme d'une page a
        // l'autre quand le decalage est pris en compte.
        let premier = recus[0].login.clone();
        if premier_login_precedent.as_ref() == Some(&premier) {
            crate::journal::alerte(
                "WebServices : le decalage semble ignore, la page 2 repete la page 1. \
                 On s'arrete a la premiere page.",
            );
            break;
        }
        premier_login_precedent = Some(premier);

        let complete = recus.len() >= taille;
        bruts.extend(recus);
        // Une page incomplete est la derniere : inutile d'en demander une
        // de plus pour se l'entendre dire.
        if !complete {
            break;
        }
    }

    let recus = bruts.len();
    let liste = serveurs::convertir(bruts);
    crate::journal::ecrire(&format!(
        "WebServices : {recus} serveur(s) recu(s) en {pages} page(s), {} en environnement {}.",
        liste.len(),
        serveurs::ENVIRONNEMENT
    ));
    Ok(liste)
}

async fn une_page(client: &reqwest::Client, url: &str) -> Result<Vec<ServeurApi>, String> {
    let reponse = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| {
            crate::journal::erreur(&format!("Appel aux WebServices echoue : {e}"));
            "Les serveurs de ManiaPlanet n'ont pas repondu.".to_string()
        })?;

    let code = reponse.status();
    if !code.is_success() {
        crate::journal::erreur(&format!("WebServices : reponse {code}"));
        return Err(format!(
            "Les serveurs de ManiaPlanet ont repondu {}.",
            code.as_u16()
        ));
    }

    reponse.json().await.map_err(|e| {
        crate::journal::erreur(&format!("Reponse des WebServices illisible : {e}"));
        "La reponse des serveurs de ManiaPlanet est illisible.".to_string()
    })
}
