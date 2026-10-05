//! Coeur metier de Cairn.
//!
//! Regle de cette caisse : **aucune entree-sortie, aucune dependance a Tauri,
//! aucun code specifique a un systeme d'exploitation.** Tout ce qui est ici
//! se teste avec `cargo test` en une seconde, sans fenetre, sans reseau et
//! sans jeu installe.
//!
//! C'est la ou vit ce qui merite d'etre juste : les deux lignes de commande
//! de lancement, le nettoyage des noms de serveurs, l'analyse de la commande
//! du registre. `cairn-app` ne fait qu'appeler le systeme et poster le
//! resultat ici.

pub mod actualites;
pub mod demarrage;
pub mod historique;
pub mod installation;
pub mod inventaire;
pub mod journal;
pub mod lancement;
pub mod lisibilite;
pub mod modes;
pub mod profils;
pub mod rang;
pub mod recherche;
pub mod reglages;
pub mod serveurs;
pub mod texte;
pub mod touches;
pub mod veille;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Le numero du contrat entre le coeur et l'interface.
///
/// **A quoi ca sert.** Le lot 20 a ajoute un champ, `nom_segments`, au
/// serveur que le pont rend. L'interface a ete rechargee a chaud par
/// Vite, le binaire Rust non, et l'application a tourne une soiree avec
/// une interface qui attendait un champ qu'un moteur plus ancien ne
/// rendait pas. Resultat : la colonne des noms vide, puis sans couleurs,
/// **et pas un signal**. Une heure de recherche pour un binaire perime.
///
/// Ce numero rend ce cas bruyant. L'interface porte le numero qu'elle
/// attend, elle demande celui du moteur au demarrage, et elle le dit
/// quand les deux different. Un numero absent compte comme different :
/// c'est justement ce que rend un moteur d'avant ce mecanisme.
///
/// **A incrementer des qu'un champ ou une commande du pont change.**
/// Pas a chaque lot, pas a chaque version : a chaque changement de
/// forme de ce qui traverse le pont.
pub const CONTRAT: u32 = 7;

/// L'adresse du fil d'actualites, et celle de sa signature.
///
/// Deux fichiers et pas un, pour que la signature porte sur les octets
/// tels qu'ils sont servis. Voir `actualites::signature_valable`.
///
/// **Ce sont des emplacements reserves, pas des adresses.** Le domaine
/// `.invalid` est reserve par la RFC 2606 et ne peut jamais etre
/// enregistre ni resolu : c'est ce qui en fait un bon emplacement
/// reserve. Le premier jet portait une adresse plausible en `pages.dev`,
/// ce qui etait une erreur a deux titres : elle ressemblait a une vraie,
/// donc personne ne voyait qu'il restait quelque chose a faire, et un
/// sous-domaine `pages.dev` se reclame librement, donc n'importe qui
/// pouvait prendre celui-la. La signature aurait refuse son fil, mais on
/// ne laisse pas une adresse reclamable dans le code.
///
/// Elles se remplacent en meme temps que `CLE_ACTUALITES`, voir
/// `docs/actualites/LISEZMOI.md`.
pub const URL_ACTUALITES: &str = "https://a-remplacer.invalid/actualites.json";
pub const URL_ACTUALITES_SIGNATURE: &str = "https://a-remplacer.invalid/actualites.json.sig";

/// Vrai tant que l'adresse du fil n'a pas ete renseignee.
///
/// **« Pas encore configure » et « reseau en panne » ne sont pas la meme
/// chose**, et le premier jet les montrait pareil : l'ecran annoncait un
/// echec de recuperation alors que rien n'avait jamais ete publie. On
/// n'appelle donc meme pas le reseau dans ce cas, et l'ecran dit ce
/// qu'il faut faire au lieu de signaler une panne qui n'existe pas.
pub fn actualites_configurees() -> bool {
    !URL_ACTUALITES.contains(".invalid/")
}

/// La cle publique qui authentifie le fil d'actualites.
///
/// **CECI EST UNE CLE DE DEVELOPPEMENT, A REMPLACER AVANT LA PREMIERE
/// DIFFUSION PUBLIQUE.** Sa moitie privee est dans `outils/`, donc dans
/// le depot, donc connue de tout le monde : elle sert a fabriquer des
/// fils d'essai, pas a prouver quoi que ce soit.
///
/// La vraie cle se fabrique sur la machine de celui qui publiera, avec
/// `cargo run --example actualites -- cle`, et sa moitie privee ne doit
/// jamais traverser une conversation, un ticket ni un depot. Seule la
/// moitie publique vient ici.
pub const CLE_ACTUALITES: &str = "4f60524915ccd40d3be38dbeb7742060194208e0ab2a57f16fa2b0edf8aadd66";

/// L'adresse des WebServices de ManiaPlanet, serveurs en ligne.
///
/// `excludeLobby` ecarte les serveurs d'attente, qui ne sont pas des parties
/// et qui trompent le classement par frequentation.
/// L'adresse des serveurs en ligne, sans la pagination.
///
/// **Aucun filtre n'est demande a l'API.** Elle en propose, et le lot 19b
/// s'en servait : `onlyPublic=1` et `excludeLobby=1`. Les deux devaient
/// partir. `onlyPublic=1` ecartait purement et simplement les serveurs a
/// mot de passe, donc le cadenas de la liste n'aurait jamais rien
/// affiche. `excludeLobby=1` ecartait le matchmaking, donc Cairn ne
/// savait pas y mener.
///
/// Elle ne sait en revanche pas filtrer sur l'environnement : tout le
/// flux ManiaPlanet arrive, Trackmania compris, et le tri sur ShootMania
/// se fait chez nous, voir [`serveurs::convertir`].
pub const URL_SERVEURS: &str = concat!(
    "https://prod.live.maniaplanet.com/webservices/servers/online",
    "?orderBy=playerCount&onlyPublic=0&onlyPrivate=0&onlyLobby=0"
);

/// Le nombre de serveurs demandes par page.
pub const PAGE_SERVEURS: usize = 250;

/// Le nombre maximal de pages parcourues.
///
/// Garde-fou et non reglage : sans lui, une API qui ignorerait le
/// decalage ferait tourner la pagination sans fin.
pub const PAGES_SERVEURS_MAX: usize = 8;

/// Le nom du processus du jeu, sans extension.
pub const PROCESSUS_JEU: &str = "ManiaPlanet";

/// L'etat de l'installation, tel que l'interface le recoit au demarrage et
/// a chaque relevé.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Etat {
    /// Chemin de l'executable, absent s'il reste introuvable.
    pub chemin_jeu: Option<String>,
    /// Le jeu tourne-t-il en ce moment.
    pub jeu_lance: bool,
    pub profils: Vec<profils::Profil>,
    /// Le profil retenu, deja recoupe avec ceux qui existent vraiment.
    pub profil_actif: Option<String>,
}

/// Le systeme sur lequel l'application tourne.
///
/// La decision « Windows seul » est prise, mais elle passe par cette
/// interface plutot que d'etre repandue dans le code. Sans cette precaution
/// la decision devient irreversible sans qu'on s'en apercoive, et c'est le
/// genre de dette qu'on ne voit qu'une fois qu'elle est chere.
pub trait Plateforme {
    /// Chemin de l'executable du jeu, ou `None` s'il est introuvable.
    fn chemin_jeu(&self) -> Option<String>;
    /// Le jeu tourne-t-il en ce moment.
    fn jeu_lance(&self) -> bool;
    /// Les profils locaux declares par le jeu, du plus recemment utilise au
    /// plus ancien.
    fn profils(&self) -> Vec<profils::Profil>;
}

/// Assemble l'etat a partir de ce que la plateforme sait dire.
///
/// Tout est ici plutot que dans la commande Tauri, pour que le recoupement
/// du profil memorise avec les profils reels soit teste.
pub fn etat<P: Plateforme>(plateforme: &P, profil_memorise: Option<&str>) -> Etat {
    let profils = plateforme.profils();
    let profil_actif = profils::profil_actif(&profils, profil_memorise);
    Etat {
        chemin_jeu: plateforme.chemin_jeu(),
        jeu_lance: plateforme.jeu_lance(),
        profils,
        profil_actif,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PlateformeFictive {
        profils: Vec<profils::Profil>,
        lance: bool,
    }

    impl Plateforme for PlateformeFictive {
        fn chemin_jeu(&self) -> Option<String> {
            Some(r"C:\MP\ManiaPlanet.exe".to_string())
        }
        fn jeu_lance(&self) -> bool {
            self.lance
        }
        fn profils(&self) -> Vec<profils::Profil> {
            self.profils.clone()
        }
    }

    fn profil(id: &str) -> profils::Profil {
        profils::Profil {
            id: id.to_string(),
            nom: id.to_string(),
            derniere_utilisation: None,
            teinte: profils::teinte(id).to_string(),
            initiale: profils::initiale(id),
        }
    }

    #[test]
    fn la_version_est_renseignee() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with("0."));
    }

    #[test]
    fn l_url_des_serveurs_ne_filtre_rien_en_amont() {
        assert!(URL_SERVEURS.starts_with("https://"));
        // Aucun filtre serveur : le cadenas et le matchmaking en
        // dependent, voir le commentaire de la constante.
        assert!(!URL_SERVEURS.contains("onlyPublic=1"));
        assert!(!URL_SERVEURS.contains("excludeLobby=1"));
        assert!(!URL_SERVEURS.contains("length="));
    }

    #[test]
    fn l_etat_recoupe_le_profil_memorise_avec_les_profils_reels() {
        let p = PlateformeFictive {
            profils: vec![profil("A"), profil("B")],
            lance: false,
        };
        assert_eq!(etat(&p, Some("B")).profil_actif.unwrap(), "B");
        // Un profil supprime dans le jeu ne doit pas rester actif.
        assert_eq!(etat(&p, Some("Disparu")).profil_actif.unwrap(), "A");
    }

    #[test]
    fn sans_profil_l_etat_reste_coherent() {
        let p = PlateformeFictive {
            profils: vec![],
            lance: true,
        };
        let e = etat(&p, Some("A"));
        assert!(e.profils.is_empty());
        assert_eq!(e.profil_actif, None);
        assert!(e.jeu_lance);
    }
}
