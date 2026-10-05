//! La construction des lignes de commande de lancement.
//!
//! Les deux chemins ont ete valides sur la machine de Flow le 2026-09-08,
//! puis a nouveau par la passerelle PowerShell du lot 18. Ce module les
//! reproduit **au caractere pres**, parce que le jeu analyse sa ligne de
//! commande a la main et pardonne tres mal.
//!
//! Point important pour l'appelant : ce module rend une **ligne brute**, pas
//! une liste d'arguments. Sous Windows, `std::process::Command::arg` applique
//! son propre echappement, qui transformerait `/url="maniaplanet://..."` en
//! `"/url=\"maniaplanet://...\""` et le jeu ignorerait l'argument en silence.
//! L'appelant doit passer cette ligne par `CommandExt::raw_arg`.

use serde::{Deserialize, Serialize};

/// Rejoindre la partie, ou seulement la regarder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Rejoindre,
    Observer,
}

impl Action {
    /// Le mot que le jeu attend dans sa ligne de commande.
    fn mot(self) -> &'static str {
        match self {
            Action::Rejoindre => "join",
            Action::Observer => "spectate",
        }
    }

    /// `true` si on ne fait que regarder.
    pub fn depuis_spectate(spectate: bool) -> Self {
        if spectate {
            Action::Observer
        } else {
            Action::Rejoindre
        }
    }
}

/// Ce qui empeche un lancement, avant meme d'avoir tente quoi que ce soit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refus {
    /// Le login du serveur est vide ou contient autre chose que les
    /// caracteres attendus.
    LoginInvalide(String),
    /// L'identifiant de titre est vide ou mal forme.
    TitreInvalide(String),
    /// Le nom de profil contient un caractere qui n'a rien a y faire.
    ProfilInvalide(String),
}

impl std::fmt::Display for Refus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refus::LoginInvalide(v) => write!(f, "login de serveur invalide : {v:?}"),
            Refus::TitreInvalide(v) => write!(f, "identifiant de titre invalide : {v:?}"),
            Refus::ProfilInvalide(v) => write!(f, "nom de profil invalide : {v:?}"),
        }
    }
}

impl std::error::Error for Refus {}

/// Ce qu'il faut executer, et comment lire le resultat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lancement {
    /// La ligne de commande brute, a passer telle quelle a `raw_arg`.
    pub ligne: String,
    /// `true` quand le jeu tournait deja : le processus qu'on demarre va
    /// passer la main a l'instance existante puis s'arreter de lui meme.
    /// Un code de sortie non nul n'est alors pas une erreur.
    pub relais: bool,
}

/// Prepare le lancement pour un serveur donne.
///
/// `profil` n'est utilise que quand le jeu est ferme : une instance qui
/// tourne deja a son profil, on ne le change pas sous ses pieds.
/// Prepare l'ouverture du jeu sur ses propres menus.
///
/// C'est le seul endroit de Cairn ou l'on veut les ecrans natifs : les
/// reglages graphiques, audio, reseau et touches vivent dans des
/// fichiers binaires qu'on ne peut pas toucher, donc la seule facon de
/// les changer est d'aller dans le jeu.
///
/// La ligne est donc l'inverse de celle de [`preparer`] : un profil, et
/// surtout **ni `/join` ni `/silent`**. `/silent` saute justement les
/// ecrans natifs, ce qui rendrait la commande sans objet.
pub fn ouvrir(profil: &str) -> Result<Lancement, Refus> {
    let profil = verifier_profil(profil)?;
    Ok(Lancement {
        ligne: proteger(&format!("/profile={profil}")),
        relais: false,
    })
}

/// Prepare l'ouverture du jeu sur le menu d'un title pack.
///
/// C'est l'action des tuiles de l'ecran Editer. Elle merite d'exister
/// separement de [`ouvrir`] pour une raison precise : le jeu **ne sait
/// pas** s'ouvrir sur son editeur de maps. Aucun drapeau de ligne de
/// commande ne le permet, et le protocole `maniaplanet://` non plus. Ce
/// qu'on peut faire de mieux est de poser le joueur sur le menu du titre
/// qu'il veut editer, d'ou l'editeur est a portee, au lieu de l'ouvrir
/// sur l'accueil et de le laisser chercher.
///
/// Surtout **pas de `/silent`** : il saute les ecrans natifs, ce qui
/// rendrait la commande sans objet, exactement comme pour [`ouvrir`].
///
/// Jeu ferme, `/title=` restreint le client au titre demande et l'ouvre
/// sur son menu. Jeu ouvert, on passe par `#menutitle=`, qui fait
/// basculer l'instance existante sans en demarrer une seconde.
pub fn ouvrir_titre(profil: &str, titre: &str, jeu_lance: bool) -> Result<Lancement, Refus> {
    let titre = verifier_titre(titre)?;

    if jeu_lance {
        let uri = format!("maniaplanet://#menutitle={titre}");
        return Ok(Lancement {
            ligne: format!("/useexedir /singleinst /url=\"{uri}\""),
            relais: true,
        });
    }

    let profil = verifier_profil(profil)?;
    Ok(Lancement {
        ligne: [
            proteger(&format!("/profile={profil}")),
            format!("/title={titre}"),
        ]
        .join(" "),
        relais: false,
    })
}

pub fn preparer(
    profil: &str,
    login: &str,
    titre: &str,
    action: Action,
    jeu_lance: bool,
) -> Result<Lancement, Refus> {
    let login = verifier_login(login)?;
    let titre = verifier_titre(titre)?;
    let cible = format!("{login}@{titre}");

    if jeu_lance {
        // Le jeu tourne deja. On reproduit exactement la commande que le
        // registre associe au protocole maniaplanet:// :
        //   "ManiaPlanet.exe" /useexedir /singleinst /url="%1"
        // Ce n'est pas Windows qui redirige vers l'instance existante, c'est
        // /singleinst. Les guillemets autour de l'URL sont LITTERAUX et
        // obligatoires, sans eux l'argument est ignore sans un mot.
        let uri = format!("maniaplanet://#{}={}", action.mot(), cible);
        return Ok(Lancement {
            ligne: format!("/useexedir /singleinst /url=\"{uri}\""),
            relais: true,
        });
    }

    // Jeu ferme : lancement direct et silencieux, aucun ecran natif.
    let profil = verifier_profil(profil)?;
    let ligne = [
        proteger(&format!("/profile={profil}")),
        format!("/{}={}", action.mot(), cible),
        "/silent".to_string(),
    ]
    .join(" ");

    Ok(Lancement {
        ligne,
        relais: false,
    })
}

/// Entoure de guillemets si, et seulement si, il y a un espace.
///
/// C'est exactement ce que faisait `Start-Process -ArgumentList` dans la
/// passerelle, et c'est donc la forme qui a ete validee sur le poste.
fn proteger(argument: &str) -> String {
    if argument.contains(' ') {
        format!("\"{argument}\"")
    } else {
        argument.to_string()
    }
}

/// Les caracteres admis dans un login de serveur ou un identifiant de titre.
///
/// Ce controle n'est pas cosmetique. Le login vient du reseau, il est ecrit
/// par le proprietaire du serveur, et il finit dans une ligne de commande
/// brute. Sans ce filtre, un login contenant un guillemet permettrait de
/// refermer l'argument et d'en ajouter d'autres. On refuse tout ce qui n'est
/// pas un caractere de login.
fn caractere_admis(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
}

fn verifier_login(login: &str) -> Result<&str, Refus> {
    let login = login.trim();
    if login.is_empty() || !login.chars().all(caractere_admis) {
        return Err(Refus::LoginInvalide(login.to_string()));
    }
    Ok(login)
}

/// L'identifiant de titre est lui meme de la forme `nom@auteur`.
///
/// Il y a donc deux `@` dans la cible finale, `login@nom@auteur`, et c'est
/// normal : le jeu decoupe sur le premier.
fn verifier_titre(titre: &str) -> Result<&str, Refus> {
    let titre = titre.trim();
    if titre.is_empty() {
        return Err(Refus::TitreInvalide(titre.to_string()));
    }
    let bon = titre.chars().all(|c| caractere_admis(c) || c == '@')
        && titre.matches('@').count() == 1
        && !titre.starts_with('@')
        && !titre.ends_with('@');
    if !bon {
        return Err(Refus::TitreInvalide(titre.to_string()));
    }
    Ok(titre)
}

/// Le profil vient de la machine, pas du reseau, mais il finit lui aussi
/// dans la ligne brute. Les espaces sont admis, ils sont proteges plus haut.
/// Le guillemet et les caracteres de separation, non.
fn verifier_profil(profil: &str) -> Result<&str, Refus> {
    let profil = profil.trim();
    if profil.is_empty()
        || profil
            .chars()
            .any(|c| matches!(c, '"' | '\r' | '\n' | '\t'))
    {
        return Err(Refus::ProfilInvalide(profil.to_string()));
    }
    Ok(profil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ouvrir_un_titre_jeu_ferme_restreint_le_client_sans_silence() {
        let l = ouvrir_titre("Xirdrak", "obstacle@smokegun", false).unwrap();
        assert_eq!(l.ligne, "/profile=Xirdrak /title=obstacle@smokegun");
        assert!(!l.relais);
        // /silent sauterait justement les ecrans natifs, ce qui rendrait
        // la commande sans objet.
        assert!(!l.ligne.contains("/silent"));
        assert!(!l.ligne.contains("/join"));
    }

    #[test]
    fn ouvrir_un_titre_jeu_ouvert_passe_par_le_protocole() {
        let l = ouvrir_titre("Xirdrak", "SMStormElite@nadeolabs", true).unwrap();
        assert_eq!(
            l.ligne,
            "/useexedir /singleinst /url=\"maniaplanet://#menutitle=SMStormElite@nadeolabs\""
        );
        assert!(l.relais);
    }

    #[test]
    fn ouvrir_un_titre_protege_un_profil_a_espaces() {
        let l = ouvrir_titre("Mon Profil", "TMAll@domino54", false).unwrap();
        assert_eq!(l.ligne, "\"/profile=Mon Profil\" /title=TMAll@domino54");
    }

    #[test]
    fn ouvrir_un_titre_refuse_un_titre_mal_forme() {
        // Le titre vient du nom d'un fichier sur le disque, donc de
        // quelque chose que n'importe qui peut renommer. Le controle est
        // le meme que pour un lancement sur serveur, et il vaut aussi
        // quand le jeu tourne deja : le titre part dans les deux cas.
        for titre in [
            "",
            "sans-arobase",
            "deux@arobas@es",
            "@debut",
            "fin@",
            "a b@c",
            "a\"@c",
        ] {
            for jeu_lance in [false, true] {
                assert!(
                    ouvrir_titre("Xirdrak", titre, jeu_lance).is_err(),
                    "{titre:?} aurait du etre refuse"
                );
            }
        }
    }

    const TITRE: &str = "SMStormElite@nadeolabs";

    #[test]
    fn jeu_ferme_lancement_direct_et_silencieux() {
        let l = preparer("Xirdrak", "mon-serveur", TITRE, Action::Rejoindre, false).unwrap();
        assert_eq!(
            l.ligne,
            "/profile=Xirdrak /join=mon-serveur@SMStormElite@nadeolabs /silent"
        );
        assert!(!l.relais);
    }

    #[test]
    fn jeu_lance_passage_par_le_protocole_avec_guillemets_litteraux() {
        let l = preparer("Xirdrak", "mon-serveur", TITRE, Action::Rejoindre, true).unwrap();
        assert_eq!(
            l.ligne,
            "/useexedir /singleinst /url=\"maniaplanet://#join=mon-serveur@SMStormElite@nadeolabs\""
        );
        assert!(l.relais);
        // Le detail qui fait tout echouer si on l'oublie.
        assert!(l.ligne.contains("/url=\""));
        assert!(l.ligne.ends_with('"'));
    }

    #[test]
    fn observer_remplace_join_par_spectate_dans_les_deux_chemins() {
        let ferme = preparer("X", "srv", TITRE, Action::Observer, false).unwrap();
        assert!(ferme.ligne.contains("/spectate=srv@"));
        let lance = preparer("X", "srv", TITRE, Action::Observer, true).unwrap();
        assert!(lance.ligne.contains("#spectate=srv@"));
    }

    #[test]
    fn le_profil_n_intervient_pas_quand_le_jeu_tourne_deja() {
        // Une instance en cours a son profil, on ne le change pas sous ses
        // pieds. Un profil vide ne doit donc pas empecher de rejoindre.
        let l = preparer("", "srv", TITRE, Action::Rejoindre, true).unwrap();
        assert!(!l.ligne.contains("profile"));
    }

    #[test]
    fn un_profil_a_espaces_est_protege_comme_le_faisait_powershell() {
        let l = preparer("Mon Profil", "srv", TITRE, Action::Rejoindre, false).unwrap();
        assert!(l.ligne.starts_with("\"/profile=Mon Profil\" "));
    }

    #[test]
    fn un_login_de_serveur_hostile_est_refuse_avant_tout_lancement() {
        // Le scenario qu'on ne veut pas : un proprietaire de serveur qui
        // referme l'argument et en ajoute d'autres.
        for mechant in [
            "srv\" /autrechose \"",
            "srv /silent",
            "srv&calc",
            "srv|calc",
            "",
            "   ",
        ] {
            assert!(
                matches!(
                    preparer("X", mechant, TITRE, Action::Rejoindre, true),
                    Err(Refus::LoginInvalide(_))
                ),
                "ce login aurait du etre refuse : {mechant:?}"
            );
        }
    }

    #[test]
    fn un_titre_mal_forme_est_refuse() {
        for mechant in ["", "sansarobase", "deux@arobases@ici", "@auteur", "nom@"] {
            assert!(
                matches!(
                    preparer("X", "srv", mechant, Action::Rejoindre, false),
                    Err(Refus::TitreInvalide(_))
                ),
                "ce titre aurait du etre refuse : {mechant:?}"
            );
        }
    }

    #[test]
    fn un_profil_avec_un_guillemet_est_refuse() {
        assert!(matches!(
            preparer("Mon\"Profil", "srv", TITRE, Action::Rejoindre, false),
            Err(Refus::ProfilInvalide(_))
        ));
    }

    #[test]
    fn la_cible_porte_bien_deux_arobases() {
        let l = preparer("X", "srv", TITRE, Action::Rejoindre, false).unwrap();
        assert_eq!(l.ligne.matches('@').count(), 2);
    }

    #[test]
    fn depuis_spectate_traduit_le_drapeau_de_l_interface() {
        assert_eq!(Action::depuis_spectate(true), Action::Observer);
        assert_eq!(Action::depuis_spectate(false), Action::Rejoindre);
    }

    #[test]
    fn ouvrir_le_jeu_ne_porte_ni_join_ni_silent() {
        let l = ouvrir("Xirdrak").unwrap();
        assert_eq!(l.ligne, "/profile=Xirdrak");
        assert!(!l.relais);
        // Les deux absences qui font tout l'interet de cette commande :
        // avec /silent le jeu sauterait les ecrans natifs, avec /join il
        // partirait sur un serveur au lieu de s'ouvrir sur ses menus.
        assert!(!l.ligne.contains("/silent"));
        assert!(!l.ligne.contains("/join"));
    }

    #[test]
    fn ouvrir_protege_un_profil_qui_porte_un_espace() {
        let l = ouvrir("Mon Profil 2").unwrap();
        assert_eq!(l.ligne, "\"/profile=Mon Profil 2\"");
    }

    #[test]
    fn ouvrir_refuse_ce_que_preparer_refuse() {
        // Le meme controle des deux cotes : un nom de profil part dans
        // une ligne de commande brute, quelle que soit la commande.
        assert!(matches!(
            ouvrir("Xirdrak\" /c calc"),
            Err(Refus::ProfilInvalide(_))
        ));
        assert!(matches!(ouvrir(""), Err(Refus::ProfilInvalide(_))));
    }
}
