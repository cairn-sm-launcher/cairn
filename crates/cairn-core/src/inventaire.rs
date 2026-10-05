//! Ce que Cairn sait des fichiers du joueur, sans en ouvrir aucun.
//!
//! L'ecran Editer repose entierement sur ce module. Trois sortes de
//! fichiers, les maps, les replays et les title packs, et pour chacune un
//! resume : combien, quelle place, quand pour la derniere fois, et
//! comment ca se repartit entre les sous-dossiers du jeu.
//!
//! **Rien n'est ouvert, jamais.** On ne lit que ce que le systeme de
//! fichiers donne deja pour lister un dossier : un nom, une taille, une
//! date. Aucun `.Gbx` n'est analyse. C'est une decision et pas une
//! paresse : un analyseur de format binaire est une surface d'attaque,
//! et le seul renseignement qu'on voulait vraiment de l'interieur des
//! fichiers, l'identifiant de titre d'un pack, est ecrit en clair dans
//! son nom de fichier. `obstacle@smokegun.Title.Pack.gbx` dit tout.
//!
//! **Aucun chemin ne sort d'ici.** Les entrees portent un nom de fichier
//! et un nom de sous-dossier, jamais un chemin absolu. C'est ce qui
//! permet a l'interface d'afficher l'inventaire sans qu'un chemin du
//! disque de quelqu'un traverse le pont et se retrouve dans une page.

use serde::{Deserialize, Serialize};

/// Combien de fichiers recents on garde par sorte.
///
/// Six, parce que la fiche depliee d'une tuile doit rester une fiche. La
/// question a laquelle elle repond est « ou en etais-je », pas « montre
/// moi tout », et pour la seconde il y a le bouton qui ouvre le dossier.
pub const RECENTS: usize = 6;

/// Les trois sortes de fichiers que l'ecran Editer sait resumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sorte {
    Maps,
    Replays,
    Packs,
}

impl Sorte {
    /// La fin de nom de fichier qui designe cette sorte, en minuscules.
    ///
    /// Le jeu n'est pas constant sur la casse, `Map.Gbx` et `Title.Pack.gbx`
    /// cohabitent dans le meme dossier chez le meme joueur. La comparaison
    /// se fait donc toujours en minuscules.
    pub fn suffixe(self) -> &'static str {
        match self {
            Sorte::Maps => ".map.gbx",
            Sorte::Replays => ".replay.gbx",
            Sorte::Packs => ".title.pack.gbx",
        }
    }

    /// Le sous-dossier de `Documents/ManiaPlanet` ou le jeu les range.
    pub fn dossier(self) -> &'static str {
        match self {
            Sorte::Maps => "Maps",
            Sorte::Replays => "Replays",
            Sorte::Packs => "Packs",
        }
    }

    /// La cle que l'interface envoie pour demander l'ouverture du dossier.
    ///
    /// Une cle et pas un chemin, et c'est tout l'interet : l'interface ne
    /// sait pas ou sont les fichiers, elle ne peut donc pas demander
    /// l'ouverture d'autre chose. Il n'y a rien a valider parce qu'il n'y
    /// a rien a contourner.
    pub fn cle(self) -> &'static str {
        match self {
            Sorte::Maps => "maps",
            Sorte::Replays => "replays",
            Sorte::Packs => "packs",
        }
    }

    /// La sorte designee par une cle, ou rien si elle n'en designe aucune.
    pub fn depuis_cle(cle: &str) -> Option<Self> {
        match cle {
            "maps" => Some(Sorte::Maps),
            "replays" => Some(Sorte::Replays),
            "packs" => Some(Sorte::Packs),
            _ => None,
        }
    }
}

/// Un fichier tel que le systeme le donne, avant tout traitement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entree {
    /// Le nom du fichier seul, extension comprise, sans aucun chemin.
    pub nom: String,
    /// Le sous-dossier qui le porte, vide quand il est a la racine.
    pub groupe: String,
    pub octets: u64,
    /// Secondes depuis 1970.
    pub modifie: u64,
}

/// Un fichier prêt a afficher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    /// Le nom sans son extension, qui est ce que le joueur a tape.
    pub nom: String,
    pub groupe: String,
    pub octets: u64,
    pub modifie: u64,
    /// L'identifiant de titre, pour un pack et pour lui seul.
    pub titre: Option<String>,
}

/// Un sous-dossier et ce qu'il pese.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Groupe {
    /// Le nom du sous-dossier, ou la chaine vide pour la racine.
    pub nom: String,
    pub nombre: usize,
    pub octets: u64,
}

/// Le resume d'une sorte de fichiers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventaire {
    pub nombre: usize,
    pub octets: u64,
    /// La date du plus recent, ou rien quand il n'y a aucun fichier.
    pub dernier: Option<u64>,
    /// Les sous-dossiers, du plus fourni au moins fourni.
    pub groupes: Vec<Groupe>,
    /// Les [`RECENTS`] fichiers les plus recemment touches.
    pub recents: Vec<Element>,
}

/// Retire le suffixe de la sorte, quelle que soit la casse.
///
/// Rend `None` quand le nom ne porte pas ce suffixe, ce qui est la facon
/// dont on ecarte tout le reste : un `.MatchSettings.txt`, une vignette,
/// un fichier de travail. Un nom qui n'est QUE le suffixe est ecarte lui
/// aussi, il ne reste rien a afficher.
pub fn sans_suffixe(sorte: Sorte, nom: &str) -> Option<&str> {
    let suffixe = sorte.suffixe();
    if nom.len() <= suffixe.len() {
        return None;
    }
    let (debut, fin) = nom.split_at(nom.len() - suffixe.len());
    if !fin.eq_ignore_ascii_case(suffixe) {
        return None;
    }
    let debut = debut.trim();
    if debut.is_empty() {
        None
    } else {
        Some(debut)
    }
}

/// L'identifiant de titre porte par le nom d'un fichier de pack.
///
/// **Ce qui sort d'ici finit dans une ligne de commande**, par
/// `maniaplanet://#menutitle=`. Le controle est donc le meme que celui de
/// [`crate::lancement`] et pas une version allegee : alphanumerique ASCII
/// plus `_`, `-` et `.`, exactement un `@`, et pas au bord. Un fichier
/// depose la par quelqu'un d'autre, ou renomme a la main, ne passe pas,
/// et le pack s'affiche alors sans bouton d'ouverture plutot que de
/// devenir un moyen de glisser autre chose dans la commande.
pub fn titre_de_pack(nom: &str) -> Option<String> {
    let titre = sans_suffixe(Sorte::Packs, nom)?;
    let bon = !titre.is_empty()
        && titre
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '@'))
        && titre.matches('@').count() == 1
        && !titre.starts_with('@')
        && !titre.ends_with('@');
    bon.then(|| titre.to_string())
}

/// Resume une liste d'entrees deja lue sur le disque.
///
/// Les entrees qui ne sont pas de la sorte demandee sont ecartees en
/// silence : un dossier de maps contient aussi des `MatchSettings`, et un
/// dossier de replays des fantomes de MediaTracker.
pub fn resumer(sorte: Sorte, entrees: &[Entree]) -> Inventaire {
    let mut elements: Vec<Element> = entrees
        .iter()
        .filter_map(|e| {
            let nom = sans_suffixe(sorte, &e.nom)?;
            Some(Element {
                nom: nom.to_string(),
                groupe: e.groupe.clone(),
                octets: e.octets,
                modifie: e.modifie,
                titre: match sorte {
                    Sorte::Packs => titre_de_pack(&e.nom),
                    _ => None,
                },
            })
        })
        .collect();

    let nombre = elements.len();
    let octets = elements.iter().map(|e| e.octets).sum();
    let dernier = elements.iter().map(|e| e.modifie).max();

    let mut groupes: Vec<Groupe> = Vec::new();
    for element in &elements {
        match groupes.iter_mut().find(|g| g.nom == element.groupe) {
            Some(groupe) => {
                groupe.nombre += 1;
                groupe.octets += element.octets;
            }
            None => groupes.push(Groupe {
                nom: element.groupe.clone(),
                nombre: 1,
                octets: element.octets,
            }),
        }
    }
    // Du plus fourni au moins fourni, alphabetique a egalite. Le tri
    // alphabetique de secours n'est pas une coquetterie : sans lui, deux
    // dossiers de meme taille changeraient de place d'une lecture a
    // l'autre, et une liste qui bouge toute seule se relit a chaque fois.
    groupes.sort_by(|a, b| b.nombre.cmp(&a.nombre).then_with(|| a.nom.cmp(&b.nom)));

    // Les plus recents d'abord. Le nom departage a date egale, ce qui
    // arrive vraiment : une copie de dossier pose la meme date sur tout.
    elements.sort_by(|a, b| b.modifie.cmp(&a.modifie).then_with(|| a.nom.cmp(&b.nom)));
    elements.truncate(RECENTS);

    Inventaire {
        nombre,
        octets,
        dernier,
        groupes,
        recents: elements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entree(nom: &str, groupe: &str, octets: u64, modifie: u64) -> Entree {
        Entree {
            nom: nom.to_string(),
            groupe: groupe.to_string(),
            octets,
            modifie,
        }
    }

    #[test]
    fn le_suffixe_se_lit_quelle_que_soit_la_casse() {
        // Les deux casses cohabitent chez le meme joueur, dans le meme
        // dossier : « Start.Map.Gbx » et « obstacle@smokegun.Title.Pack.gbx ».
        assert_eq!(sans_suffixe(Sorte::Maps, "Start.Map.Gbx"), Some("Start"));
        assert_eq!(sans_suffixe(Sorte::Maps, "Start.map.gbx"), Some("Start"));
        assert_eq!(sans_suffixe(Sorte::Maps, "Start.MAP.GBX"), Some("Start"));
        assert_eq!(
            sans_suffixe(Sorte::Replays, "Chinatown_flow.Replay.Gbx"),
            Some("Chinatown_flow")
        );
    }

    #[test]
    fn ce_qui_n_est_pas_de_la_sorte_est_ecarte() {
        assert_eq!(sans_suffixe(Sorte::Maps, "Start.Replay.Gbx"), None);
        assert_eq!(sans_suffixe(Sorte::Maps, "liste.MatchSettings.txt"), None);
        assert_eq!(sans_suffixe(Sorte::Maps, "Start.Map"), None);
        // Un nom qui n'est que le suffixe ne laisse rien a afficher.
        assert_eq!(sans_suffixe(Sorte::Maps, ".Map.Gbx"), None);
        assert_eq!(sans_suffixe(Sorte::Maps, "  .Map.Gbx"), None);
    }

    #[test]
    fn un_replay_n_est_pas_une_map() {
        // Les deux suffixes se terminent pareil, seule la partie du
        // milieu les separe. Une comparaison trop courte les confondrait.
        assert_eq!(sans_suffixe(Sorte::Replays, "x.Map.Gbx"), None);
        assert_eq!(sans_suffixe(Sorte::Packs, "x.Map.Gbx"), None);
        assert_eq!(sans_suffixe(Sorte::Maps, "x@y.Title.Pack.gbx"), None);
    }

    #[test]
    fn le_titre_d_un_pack_se_lit_dans_son_nom() {
        assert_eq!(
            titre_de_pack("obstacle@smokegun.Title.Pack.gbx").as_deref(),
            Some("obstacle@smokegun")
        );
        assert_eq!(
            titre_de_pack("SMStormElite@nadeolabs.Title.Pack.gbx").as_deref(),
            Some("SMStormElite@nadeolabs")
        );
        assert_eq!(
            titre_de_pack("TMAll@domino54.Title.Pack.gbx").as_deref(),
            Some("TMAll@domino54")
        );
    }

    #[test]
    fn un_nom_de_pack_bricole_ne_donne_pas_de_titre() {
        // Ce qui sort d'ici part dans une ligne de commande. Un fichier
        // renomme a la main ne doit pas pouvoir y glisser quoi que ce
        // soit, il perd simplement son bouton d'ouverture.
        for nom in [
            "sans-arobase.Title.Pack.gbx",
            "deux@arobases@ici.Title.Pack.gbx",
            "@debut.Title.Pack.gbx",
            "fin@.Title.Pack.gbx",
            "espace dedans@auteur.Title.Pack.gbx",
            "guillemet\"@auteur.Title.Pack.gbx",
            "slash/@auteur.Title.Pack.gbx",
            "accente@auteur.Title.Pack.gbx".replace('e', "é").as_str(),
        ] {
            assert_eq!(titre_de_pack(nom), None, "{nom} aurait du etre refuse");
        }
    }

    #[test]
    fn un_inventaire_vide_ne_dit_pas_de_date() {
        let vide = resumer(Sorte::Maps, &[]);
        assert_eq!(vide.nombre, 0);
        assert_eq!(vide.octets, 0);
        assert_eq!(vide.dernier, None);
        assert!(vide.groupes.is_empty());
        assert!(vide.recents.is_empty());
    }

    #[test]
    fn le_resume_compte_pese_et_date() {
        let entrees = [
            entree("Start.Map.Gbx", "My Maps", 100, 30),
            entree("BXS.Map.Gbx", "My Maps", 200, 50),
            entree("RedRock.Map.Gbx", "", 400, 10),
            // Ecartes : mauvaise sorte, et un fichier de reglages.
            entree("course.Replay.Gbx", "", 999, 999),
            entree("liste.MatchSettings.txt", "MatchSettings", 999, 999),
        ];
        let inv = resumer(Sorte::Maps, &entrees);
        assert_eq!(inv.nombre, 3);
        assert_eq!(inv.octets, 700);
        assert_eq!(inv.dernier, Some(50));
    }

    #[test]
    fn les_groupes_vont_du_plus_fourni_au_moins_fourni() {
        let entrees = [
            entree("a.Map.Gbx", "My Maps", 1, 1),
            entree("b.Map.Gbx", "My Maps", 1, 2),
            entree("c.Map.Gbx", "Downloaded", 1, 3),
            entree("d.Map.Gbx", "", 1, 4),
        ];
        let inv = resumer(Sorte::Maps, &entrees);
        assert_eq!(
            inv.groupes
                .iter()
                .map(|g| g.nom.as_str())
                .collect::<Vec<_>>(),
            ["My Maps", "", "Downloaded"]
        );
        assert_eq!(inv.groupes[0].nombre, 2);
        // A egalite, l'ordre alphabetique, pour qu'il ne bouge pas d'une
        // lecture a l'autre.
        assert_eq!(inv.groupes[1].nom, "");
        assert_eq!(inv.groupes[2].nom, "Downloaded");
    }

    #[test]
    fn les_recents_sont_les_plus_recents_et_pas_plus_de_six() {
        let entrees: Vec<Entree> = (0..20)
            .map(|i| entree(&format!("map{i}.Map.Gbx"), "", 1, i))
            .collect();
        let inv = resumer(Sorte::Maps, &entrees);
        assert_eq!(inv.nombre, 20);
        assert_eq!(inv.recents.len(), RECENTS);
        assert_eq!(inv.recents[0].nom, "map19");
        assert_eq!(inv.recents[RECENTS - 1].nom, "map14");
    }

    #[test]
    fn a_date_egale_le_nom_departage() {
        // Une copie de dossier pose la meme date sur tous les fichiers.
        // Sans ce departage, la liste changerait d'ordre a chaque lecture.
        let entrees = [
            entree("charlie.Map.Gbx", "", 1, 7),
            entree("alpha.Map.Gbx", "", 1, 7),
            entree("bravo.Map.Gbx", "", 1, 7),
        ];
        let inv = resumer(Sorte::Maps, &entrees);
        assert_eq!(
            inv.recents
                .iter()
                .map(|e| e.nom.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "bravo", "charlie"]
        );
    }

    #[test]
    fn seuls_les_packs_portent_un_titre() {
        let inv = resumer(
            Sorte::Packs,
            &[
                entree("obstacle@smokegun.Title.Pack.gbx", "", 10, 1),
                entree("bricole.Title.Pack.gbx", "", 20, 2),
            ],
        );
        assert_eq!(inv.nombre, 2);
        // Le plus recent d'abord : le pack bricole, sans titre.
        assert_eq!(inv.recents[0].nom, "bricole");
        assert_eq!(inv.recents[0].titre, None);
        assert_eq!(inv.recents[1].titre.as_deref(), Some("obstacle@smokegun"));

        let maps = resumer(Sorte::Maps, &[entree("Start.Map.Gbx", "", 1, 1)]);
        assert_eq!(maps.recents[0].titre, None);
    }

    #[test]
    fn les_cles_font_l_aller_et_le_retour() {
        for sorte in [Sorte::Maps, Sorte::Replays, Sorte::Packs] {
            assert_eq!(Sorte::depuis_cle(sorte.cle()), Some(sorte));
        }
        // Tout le reste est refuse, y compris ce qui ressemble a un chemin.
        for cle in ["", "Maps", "MAPS", "..", "../Config", "C:\\", "profils"] {
            assert_eq!(Sorte::depuis_cle(cle), None, "{cle} aurait du etre refuse");
        }
    }
}
