//! La liste des serveurs en ligne.
//!
//! L'appel reseau vit dans `cairn-app`. Ce qui est ici, c'est la traduction
//! de ce que rend l'API en ce que l'interface affiche, et c'est la que se
//! trouvent les pieges : des noms codes, des noms de scripts, et des champs
//! que l'API laisse absents plutot que nuls.

use serde::{Deserialize, Serialize};

use crate::{modes, texte};

/// Un serveur tel que les WebServices le rendent.
///
/// Les valeurs par defaut ne sont pas de la complaisance : l'API omet
/// franchement certains champs au lieu de les mettre a `null`, et une
/// deserialisation stricte ferait perdre toute la liste pour un serveur mal
/// renseigne.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ServeurApi {
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub script_name: String,
    #[serde(default)]
    pub environment: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub player_count: u32,
    #[serde(default)]
    pub player_max: u32,
    #[serde(default)]
    pub spectator_count: u32,
    /// Le serveur demande un mot de passe.
    #[serde(default)]
    pub is_private: bool,
    /// L'observation demande un mot de passe, meme quand le jeu ne le
    /// demande pas.
    #[serde(default)]
    pub is_spectator_private: bool,
    /// Un lobby est la porte d'entree du matchmaking : on le rejoint
    /// comme un serveur, mais il met en file jusqu'a ce qu'une partie se
    /// forme.
    #[serde(default)]
    pub is_lobby: bool,
    /// Non vide quand ce serveur n'est qu'un relais d'un autre, dont il
    /// porte ici le login. Un relais ne sert qu'a repartir la charge des
    /// spectateurs, il duplique donc une partie qui se joue ailleurs.
    #[serde(default)]
    pub relay_of: String,
    #[serde(default)]
    pub ladder_limit_min: u32,
    #[serde(default)]
    pub ladder_limit_max: u32,
}

/// Un serveur tel que l'interface le recoit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Serveur {
    /// Le nom debarrasse de ses codes de mise en forme.
    pub nom: String,
    /// Le nom brut, codes compris. Garde parce que c'est la verite de
    /// l'API, et parce qu'un rendu futur pourrait vouloir en faire
    /// autre chose que ce que fait `nom_segments`.
    pub nom_brut: String,
    /// Le nom decoupe pour l'affichage, couleurs deja ramenees a un
    /// contraste lisible. C'est ce que dessine la liste.
    ///
    /// Calcule ici et pas dans l'interface pour la raison habituelle :
    /// l'analyse des codes du jeu et le calcul de contraste sont des
    /// regles, elles se testent, donc elles vivent en Rust. Le cout est
    /// un vecteur par serveur, presque toujours d'un seul element
    /// puisque la plupart des noms ne portent aucun code.
    pub nom_segments: Vec<texte::SegmentAffiche>,
    pub mode: String,
    pub login: String,
    pub titre: String,
    pub joueurs: u32,
    pub maximum: u32,
    pub observateurs: u32,
    /// L'API ne donne aucune latence. Plutot qu'un chiffre invente, le champ
    /// reste absent et la fiche affiche un tiret.
    pub latence: Option<u32>,
    /// La description, debarrassee de ses codes comme le nom.
    pub description: String,
    /// Le pays, tire de la zone. Vide quand la zone ne le dit pas.
    pub region: String,
    /// La zone complete, telle que l'API la donne, pour la fiche detaillee.
    pub zone: String,
    /// Demande un mot de passe : c'est le cadenas de la liste.
    pub prive: bool,
    /// L'observation demande un mot de passe alors que le jeu n'en demande
    /// pas. Ca vaut la peine de le dire avant de cliquer sur Observer.
    pub observation_privee: bool,
    /// Porte d'entree du matchmaking.
    pub lobby: bool,
    /// Le login du serveur dont celui-ci n'est qu'un relais.
    pub relais: Option<String>,
    /// Les bornes de ladder, quand il y en a de vraies. Voir [`ladder`].
    pub ladder: Option<(u32, u32)>,
}

/// La borne haute de ladder que porte un serveur sans limite.
///
/// **Valeur observee, pas documentee.** Les serveurs echantillonnes
/// rendent tous `ladder_limit_min: 0` et `ladder_limit_max: 80000`, y
/// compris ceux qui n'annoncent aucune restriction. On considere donc
/// que cette paire veut dire « pas de limite ». Si des serveurs
/// reellement bornes se mettaient a disparaitre de l'affichage, c'est
/// cette constante qu'il faut corriger, et elle est seule.
pub const LADDER_SANS_LIMITE: u32 = 80_000;

/// Les bornes de ladder, ou `None` quand le serveur n'en impose pas.
pub fn ladder(min: u32, max: u32) -> Option<(u32, u32)> {
    let borne_basse = min > 0;
    let borne_haute = max > 0 && max < LADDER_SANS_LIMITE;
    (borne_basse || borne_haute).then_some((min, max))
}

/// Le pays, tire de la zone `World|Europe|France|Ile-de-France|Paris`.
///
/// On prend le troisieme segment, qui est le pays. Les deux premiers
/// sont toujours `World` et le continent, donc ils ne distinguent rien a
/// l'echelle d'une communaute europeenne, et les suivants descendent a
/// la region puis a la ville, ce qui est trop fin pour une colonne.
pub fn region(zone: &str) -> String {
    zone.split('|').nth(2).unwrap_or("").trim().to_string()
}

/// L'environnement de ShootMania. Les WebServices servent aussi Trackmania.
pub const ENVIRONNEMENT: &str = "Storm";

/// Traduit la reponse de l'API en liste affichable.
///
/// Ne garde que ShootMania, ecarte les serveurs sans login, qui ne sont
/// rejoignables par personne, et trie par frequentation decroissante.
pub fn convertir(bruts: Vec<ServeurApi>) -> Vec<Serveur> {
    let mut liste: Vec<Serveur> = bruts
        .into_iter()
        .filter(|s| s.environment == ENVIRONNEMENT)
        .filter(|s| !s.login.trim().is_empty())
        .map(convertir_un)
        .collect();

    // Tri stable : a frequentation egale, l'ordre de l'API est conserve.
    liste.sort_by_key(|s| std::cmp::Reverse(s.joueurs));
    liste
}

fn convertir_un(brut: ServeurApi) -> Serveur {
    let nom_propre = texte::nettoyer(&brut.name);
    // Le repli sur le login vaut pour les deux formes du nom : une
    // ligne dont le texte est vide est illisible et incliquable, qu'on
    // l'affiche en couleurs ou non.
    let segments = if nom_propre.is_empty() {
        vec![texte::SegmentAffiche {
            texte: brut.login.clone(),
            couleur: None,
            gras: false,
            italique: false,
        }]
    } else {
        texte::segments_affiches(&brut.name)
    };
    Serveur {
        // Un nom entierement fait de codes donne une ligne vide, illisible
        // et incliquable. Le login prend alors le relais : il est moche mais
        // il identifie.
        nom: if nom_propre.is_empty() {
            brut.login.clone()
        } else {
            nom_propre
        },
        nom_segments: segments,
        nom_brut: brut.name,
        mode: modes::lisible(&brut.script_name),
        login: brut.login,
        titre: brut.title,
        joueurs: brut.player_count,
        maximum: brut.player_max,
        observateurs: brut.spectator_count,
        latence: None,
        description: texte::nettoyer(&brut.description),
        region: region(&brut.zone),
        zone: brut.zone,
        prive: brut.is_private,
        observation_privee: brut.is_spectator_private,
        lobby: brut.is_lobby,
        relais: (!brut.relay_of.trim().is_empty()).then_some(brut.relay_of),
        ladder: ladder(brut.ladder_limit_min, brut.ladder_limit_max),
    }
}

/// Les serveurs ou il y a quelqu'un, et les autres.
///
/// La separation existe parce que la population de ShootMania est
/// faible : l'immense majorite des serveurs est a zero, et une liste
/// triee par frequentation devient une longue trainee de zeros ou l'oeil
/// se perd. Les vides ne sont pas caches pour autant, ils sont ailleurs :
/// un serveur vide reste rejoignable, et c'est souvent comme ca qu'une
/// soiree commence.
///
/// Un observateur compte : quelqu'un qui regarde est quelqu'un.
pub fn separer(serveurs: &[Serveur]) -> (Vec<Serveur>, Vec<Serveur>) {
    serveurs
        .iter()
        .cloned()
        .partition(|s| s.joueurs > 0 || s.observateurs > 0)
}

/// Sur quoi la liste est triee.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tri {
    /// Par frequentation decroissante. C'est le defaut : la question que
    /// se pose un joueur qui ouvre la liste est « ou y a-t-il du monde ».
    #[default]
    Joueurs,
    Nom,
    Mode,
}

/// Applique un tri. Toujours stable, donc a valeur egale l'ordre de
/// l'API est conserve, ce qui evite qu'une liste rafraichie se remue
/// sous les yeux sans raison.
pub fn trier(serveurs: &mut [Serveur], tri: Tri, croissant: bool) {
    trier_dans_l_ordre(serveurs, tri);
    if croissant != ordre_naturel_croissant(tri) {
        serveurs.reverse();
    }
}

/// L'ordre « naturel » de chaque critere, celui qu'on veut au premier
/// clic.
///
/// Pour la frequentation c'est le decroissant : la question que se pose
/// un joueur qui ouvre la liste est « ou y a-t-il du monde », pas « ou
/// n'y a-t-il personne ». Pour un nom ou un mode, c'est l'alphabetique.
pub fn ordre_naturel_croissant(tri: Tri) -> bool {
    !matches!(tri, Tri::Joueurs)
}

/// Ce que devient le tri quand on clique sur un critere.
///
/// Trois etats et pas deux, demande par Flow au lot 19d bis : on pouvait
/// aller de A a Z et de Z a A, mais pas revenir. Le cycle est donc
///
/// 1. premier clic sur un critere : son sens naturel ;
/// 2. deuxieme clic : le sens inverse ;
/// 3. troisieme clic : **retour au tri par defaut**, c'est a dire la
///    frequentation decroissante, qui est aussi l'ordre dans lequel
///    l'API rend la liste. C'est ca, l'etat neutre : il n'y a pas de
///    « liste non triee » a proposer, puisqu'une liste arrive toujours
///    dans un ordre, et celui la est celui d'origine.
///
/// La regle est la meme pour les trois criteres. Sur la frequentation,
/// qui est deja le tri par defaut, elle se voit comme un simple
/// va-et-vient entre decroissant et croissant, ce qui est exactement ce
/// qu'on attend d'elle.
pub fn tri_suivant(courant: Tri, croissant: bool, demande: Tri) -> (Tri, bool) {
    let naturel = ordre_naturel_croissant(demande);
    if courant != demande {
        return (demande, naturel);
    }
    if croissant == naturel {
        return (demande, !naturel);
    }
    let defaut = Tri::default();
    (defaut, ordre_naturel_croissant(defaut))
}

fn trier_dans_l_ordre(serveurs: &mut [Serveur], tri: Tri) {
    match tri {
        Tri::Joueurs => serveurs.sort_by_key(|s| std::cmp::Reverse(s.joueurs)),
        // Le tri par nom se fait sur la forme normalisee, sinon « Λtría »
        // se rangerait apres « Zulu » et personne ne comprendrait.
        Tri::Nom => serveurs.sort_by_key(|s| crate::recherche::normaliser(&s.nom)),
        Tri::Mode => serveurs.sort_by_key(|s| {
            (
                crate::recherche::normaliser(&s.mode),
                std::cmp::Reverse(s.joueurs),
            )
        }),
    }
}

/// Ce que la barre de filtres demande.
#[derive(Debug, Clone, Default)]
pub struct Filtre<'a> {
    /// `None` veut dire tous les modes.
    pub mode: Option<&'a str>,
    pub recherche: &'a str,
    /// `None` veut dire les deux, `Some(false)` ceux qu'on peut
    /// rejoindre sans mot de passe, `Some(true)` seulement les prives.
    pub prive: Option<bool>,
    /// Ne garder que les favoris.
    pub favoris_seuls: bool,
    /// Les logins mis en favori, pour le filtre ci dessus.
    pub favoris: &'a [String],
}

/// Vrai quand le serveur passe le filtre.
pub fn retenu(serveur: &Serveur, filtre: &Filtre) -> bool {
    if filtre.favoris_seuls && !filtre.favoris.iter().any(|f| f == &serveur.login) {
        return false;
    }
    if let Some(prive) = filtre.prive {
        if serveur.prive != prive {
            return false;
        }
    }
    if let Some(mode) = filtre.mode {
        if serveur.mode != mode {
            return false;
        }
    }
    crate::recherche::correspond(&champs(serveur), filtre.recherche)
}

/// Ce sur quoi la recherche porte.
///
/// Le nom **debarrasse de ses codes**, sinon un `$` taperait dans le
/// vide. Le login, parce que c'est parfois lui qu'on connait. Et le
/// mode, demande par Flow au lot 19d : avec une centaine de serveurs et
/// autant de modes communautaires, taper « obstacle » doit ramener les
/// serveurs d'Obstacle sans avoir a trouver la bonne puce.
fn champs(serveur: &Serveur) -> [&str; 3] {
    [&serveur.nom, &serveur.login, &serveur.mode]
}

pub fn filtrer(serveurs: &[Serveur], filtre: &Filtre) -> Vec<Serveur> {
    serveurs
        .iter()
        .filter(|s| retenu(s, filtre))
        .cloned()
        .collect()
}

/// Un mode de la barre de filtres, avec ce qu'il rapporterait.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ModeCompte {
    pub mode: String,
    pub compte: usize,
}

/// Les modes presents et leur compte, **la recherche appliquee mais pas
/// le mode**.
///
/// C'est tout l'interet du compte sur la puce : il dit ce qu'on
/// gagnerait en changeant de mode, pas ce qu'on a deja. Sans ca,
/// l'utilisateur tombe sur une liste vide sans comprendre lequel de ses
/// deux filtres l'a videe.
pub fn comptes_par_mode(serveurs: &[Serveur], recherche: &str) -> Vec<ModeCompte> {
    let mut comptes: Vec<ModeCompte> = Vec::new();
    for s in serveurs {
        if !crate::recherche::correspond(&champs(s), recherche) {
            continue;
        }
        match comptes.iter_mut().find(|c| c.mode == s.mode) {
            Some(c) => c.compte += 1,
            None => comptes.push(ModeCompte {
                mode: s.mode.clone(),
                compte: 1,
            }),
        }
    }
    // Du plus joue au moins joue. La communaute cree beaucoup de modes,
    // il y en a des dizaines en ligne un soir donne, et l'interface n'en
    // montre que les premiers avant de proposer le reste. Cet ordre
    // decide donc de ce qu'on voit sans cliquer, et il doit etre celui
    // de la frequentation, pas celui de l'alphabet.
    comptes.sort_by_key(|c| {
        (
            std::cmp::Reverse(c.compte),
            crate::recherche::normaliser(&c.mode),
        )
    });
    comptes
}

/// Choisit un serveur au hasard parmi des candidats, en evitant ceux qui
/// viennent d'etre proposes.
///
/// `alea` est un nombre quelconque fourni par l'appelant : le hasard
/// n'entre pas dans une caisse sans entrees-sorties, et ca rend la
/// fonction verifiable.
///
/// Si tous les candidats ont deja ete proposes, on repart sur
/// l'ensemble : mieux vaut reproposer que ne rien rendre. C'est le seul
/// endroit ou la memoire des tirages precedents est ignoree, et
/// volontairement.
pub fn tirage(nombre_de_candidats: usize, recents: &[usize], alea: u64) -> Option<usize> {
    if nombre_de_candidats == 0 {
        return None;
    }
    let frais: Vec<usize> = (0..nombre_de_candidats)
        .filter(|i| !recents.contains(i))
        .collect();
    let pioche = if frais.is_empty() {
        (0..nombre_de_candidats).collect()
    } else {
        frais
    };
    Some(pioche[(alea % pioche.len() as u64) as usize])
}

/// Tout ce dont l'ecran Parcourir les serveurs a besoin, d'un coup.
///
/// Un seul aller-retour par frappe plutot qu'un par information, et une
/// seule fonction a verifier plutot que quatre appels a coordonner dans
/// l'interface.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Vue {
    /// Les serveurs ou il y a quelqu'un, filtres et tries.
    pub peuples: Vec<Serveur>,
    /// Les autres, filtres et tries de la meme facon.
    pub vides: Vec<Serveur>,
    /// Les modes et leur compte, pour les puces de la barre de filtres.
    pub comptes: Vec<ModeCompte>,
    /// Le nombre de serveurs ShootMania connus, filtres compris.
    pub total: usize,
    /// Le nombre qui passe le filtre. La difference avec `total` est ce
    /// que l'utilisateur gagnerait en relachant ses filtres.
    pub retenus: usize,
}

pub fn vue(serveurs: &[Serveur], filtre: &Filtre, tri: Tri, croissant: bool) -> Vue {
    let mut retenus = filtrer(serveurs, filtre);
    trier(&mut retenus, tri, croissant);
    let (peuples, vides) = separer(&retenus);
    Vue {
        comptes: comptes_par_mode(serveurs, filtre.recherche),
        total: serveurs.len(),
        retenus: retenus.len(),
        peuples,
        vides,
    }
}

/// Les modes presents dans une liste, pour alimenter les filtres.
///
/// L'ordre suit la frequentation, la liste etant deja triee : le mode le
/// plus joue se presente en premier. Ca vaut mieux qu'un ordre alphabetique,
/// qui mettrait Battle avant Obstacle un soir ou personne ne joue a Battle.
pub fn modes_presents(serveurs: &[Serveur]) -> Vec<String> {
    let mut vus: Vec<String> = Vec::new();
    for s in serveurs {
        if !vus.iter().any(|m| m == &s.mode) {
            vus.push(s.mode.clone());
        }
    }
    vus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brut(login: &str, name: &str, env: &str, joueurs: u32) -> ServeurApi {
        ServeurApi {
            login: login.to_string(),
            name: name.to_string(),
            title: "SMStormElite@nadeolabs".to_string(),
            script_name: "SM_Elite_Online.Script.txt".to_string(),
            environment: env.to_string(),
            player_count: joueurs,
            player_max: 16,
            spectator_count: 0,
            ..Default::default()
        }
    }

    /// Le meme, avec tout ce qui fait une ligne de liste complete.
    fn complet(login: &str, name: &str, joueurs: u32) -> ServeurApi {
        ServeurApi {
            description: "$fffUn serveur $f00communautaire".to_string(),
            zone: "World|Europe|France|Bretagne|Rennes".to_string(),
            is_private: true,
            is_spectator_private: true,
            is_lobby: true,
            relay_of: "serveur_source".to_string(),
            ladder_limit_min: 1000,
            ladder_limit_max: 50000,
            ..brut(login, name, "Storm", joueurs)
        }
    }

    #[test]
    fn seul_shootmania_est_garde() {
        let liste = convertir(vec![
            brut("a", "A", "Storm", 5),
            brut("b", "B", "Canyon", 9),
            brut("c", "C", "Stadium", 9),
        ]);
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].login, "a");
    }

    #[test]
    fn le_tri_se_fait_sur_la_frequentation_decroissante() {
        let liste = convertir(vec![
            brut("a", "A", "Storm", 2),
            brut("b", "B", "Storm", 14),
            brut("c", "C", "Storm", 7),
        ]);
        let joueurs: Vec<u32> = liste.iter().map(|s| s.joueurs).collect();
        assert_eq!(joueurs, vec![14, 7, 2]);
    }

    #[test]
    fn les_noms_sont_nettoyes_et_le_brut_est_conserve() {
        let liste = convertir(vec![brut("a", "$o$nNac's $i$63EBedroom", "Storm", 1)]);
        assert_eq!(liste[0].nom, "Nac's Bedroom");
        assert_eq!(liste[0].nom_brut, "$o$nNac's $i$63EBedroom");
    }

    #[test]
    fn un_nom_entierement_code_retombe_sur_le_login() {
        let liste = convertir(vec![brut("serveur-42", "$fff$o$i", "Storm", 1)]);
        assert_eq!(liste[0].nom, "serveur-42");
    }

    #[test]
    fn un_serveur_sans_login_est_ecarte() {
        // Il ne serait rejoignable par personne, et il ferait une ligne
        // morte dans la liste.
        let liste = convertir(vec![
            brut("", "Fantome", "Storm", 30),
            brut("a", "A", "Storm", 1),
        ]);
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].login, "a");
    }

    #[test]
    fn la_latence_reste_absente() {
        let liste = convertir(vec![brut("a", "A", "Storm", 1)]);
        assert_eq!(liste[0].latence, None);
    }

    #[test]
    fn les_champs_absents_de_l_api_ne_font_pas_perdre_la_liste() {
        // Un serveur reel mal renseigne, tel que l'API peut le rendre.
        let json = r#"[{"login":"a","name":"A","environment":"Storm"}]"#;
        let bruts: Vec<ServeurApi> = serde_json::from_str(json).unwrap();
        let liste = convertir(bruts);
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].joueurs, 0);
        assert_eq!(liste[0].mode, "Custom");
        assert_eq!(liste[0].titre, "");
    }

    #[test]
    fn les_modes_presents_suivent_la_frequentation() {
        let mut a = brut("a", "A", "Storm", 3);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("b", "B", "Storm", 20);
        b.script_name = "SM_Royal_Online.Script.txt".into();
        let mut c = brut("c", "C", "Storm", 10);
        c.script_name = "Obstacle.Script.txt".into();

        let liste = convertir(vec![a, b, c]);
        assert_eq!(modes_presents(&liste), vec!["Royal", "Obstacle"]);
    }

    #[test]
    fn une_liste_vide_traverse_sans_incident() {
        assert!(convertir(vec![]).is_empty());
        assert!(modes_presents(&[]).is_empty());
    }

    #[test]
    fn les_champs_de_la_ligne_de_liste_arrivent_tous() {
        let liste = convertir(vec![complet("a", "$o$fffAtria", 3)]);
        let s = &liste[0];
        assert_eq!(s.nom, "Atria");
        // La description est nettoyee comme le nom, elle porte les memes
        // codes et personne ne veut lire « $fff ».
        assert_eq!(s.description, "Un serveur communautaire");
        assert_eq!(s.region, "France");
        assert_eq!(s.zone, "World|Europe|France|Bretagne|Rennes");
        assert!(s.prive);
        assert!(s.observation_privee);
        assert!(s.lobby);
        assert_eq!(s.relais.as_deref(), Some("serveur_source"));
        assert_eq!(s.ladder, Some((1000, 50000)));
    }

    #[test]
    fn un_serveur_ordinaire_ne_porte_aucun_de_ces_marqueurs() {
        let liste = convertir(vec![brut("a", "A", "Storm", 1)]);
        let s = &liste[0];
        assert!(!s.prive);
        assert!(!s.lobby);
        assert_eq!(s.relais, None);
        assert_eq!(s.ladder, None);
        assert_eq!(s.region, "");
    }

    #[test]
    fn la_region_est_le_pays_et_rien_d_autre() {
        assert_eq!(region("World|Europe|France|Ile-de-France|Paris"), "France");
        assert_eq!(region("World|America|Canada"), "Canada");
        // Une zone trop courte ou absente ne doit pas rendre « World ».
        assert_eq!(region("World|Europe"), "");
        assert_eq!(region("World"), "");
        assert_eq!(region(""), "");
    }

    #[test]
    fn le_ladder_par_defaut_ne_compte_pas_pour_une_limite() {
        // La paire observee sur les serveurs sans restriction.
        assert_eq!(ladder(0, LADDER_SANS_LIMITE), None);
        assert_eq!(ladder(0, 0), None);
        // Une vraie borne, d'un cote ou de l'autre.
        assert_eq!(ladder(0, 50_000), Some((0, 50_000)));
        assert_eq!(ladder(20_000, LADDER_SANS_LIMITE), Some((20_000, 80_000)));
    }

    #[test]
    fn les_peuples_et_les_vides_se_separent_en_gardant_leur_ordre() {
        let liste = convertir(vec![
            brut("a", "A", "Storm", 0),
            brut("b", "B", "Storm", 7),
            brut("c", "C", "Storm", 0),
            brut("d", "D", "Storm", 2),
        ]);
        let (peuples, vides) = separer(&liste);
        assert_eq!(
            peuples.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
            vec!["b", "d"]
        );
        assert_eq!(
            vides.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
            vec!["a", "c"]
        );
    }

    #[test]
    fn un_serveur_sans_joueur_mais_avec_un_observateur_n_est_pas_vide() {
        // Quelqu'un qui regarde est quelqu'un : la ligne a sa place en
        // haut, pas dans le repli des serveurs deserts.
        let mut avec = brut("a", "A", "Storm", 0);
        avec.spectator_count = 1;
        let liste = convertir(vec![avec]);
        let (peuples, vides) = separer(&liste);
        assert_eq!(peuples.len(), 1);
        assert!(vides.is_empty());
    }

    #[test]
    fn le_tri_par_nom_ignore_les_accents() {
        let liste = convertir(vec![
            brut("a", "Zulu", "Storm", 9),
            brut("b", "Λtría", "Storm", 1),
            brut("c", "Elite", "Storm", 5),
        ]);
        let mut trie = liste.clone();
        trier(&mut trie, Tri::Nom, true);
        let noms: Vec<&str> = trie.iter().map(|s| s.nom.as_str()).collect();
        // Sans normalisation, « Λtría » se rangerait apres « Zulu ».
        assert_eq!(noms, vec!["Λtría", "Elite", "Zulu"]);
    }

    #[test]
    fn le_tri_par_mode_range_les_plus_peuples_en_tete_de_chaque_mode() {
        let mut a = brut("a", "A", "Storm", 1);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("b", "B", "Storm", 9);
        b.script_name = "Obstacle.Script.txt".into();
        let mut c = brut("c", "C", "Storm", 5);
        c.script_name = "SM_Elite_Online.Script.txt".into();

        let mut liste = convertir(vec![a, b, c]);
        trier(&mut liste, Tri::Mode, true);
        let ordre: Vec<&str> = liste.iter().map(|s| s.login.as_str()).collect();
        assert_eq!(ordre, vec!["c", "b", "a"]);
    }

    #[test]
    fn le_filtre_croise_le_mode_et_la_recherche() {
        let mut a = brut("atria_obs", "Atria - Obstacle", "Storm", 3);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("atria_elite", "Atria - Elite", "Storm", 2);
        b.script_name = "SM_Elite_Online.Script.txt".into();
        let mut c = brut("nac", "Nac's Bedroom", "Storm", 1);
        c.script_name = "Obstacle.Script.txt".into();
        let liste = convertir(vec![a, b, c]);

        let tout = filtrer(&liste, &Filtre::default());
        assert_eq!(tout.len(), 3);

        let obstacle = filtrer(
            &liste,
            &Filtre {
                mode: Some("Obstacle"),
                ..Filtre::default()
            },
        );
        assert_eq!(obstacle.len(), 2);

        let croise = filtrer(
            &liste,
            &Filtre {
                mode: Some("Obstacle"),
                recherche: "atria",
                ..Filtre::default()
            },
        );
        assert_eq!(croise.len(), 1);
        assert_eq!(croise[0].login, "atria_obs");
    }

    #[test]
    fn le_compte_des_puces_applique_la_recherche_mais_pas_le_mode() {
        // C'est tout l'interet du compte : il dit ce qu'on gagnerait en
        // changeant de mode, sinon la puce active afficherait son propre
        // total et les autres zero.
        let mut a = brut("atria_obs", "Atria - Obstacle", "Storm", 3);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("atria_elite", "Atria - Elite", "Storm", 2);
        b.script_name = "SM_Elite_Online.Script.txt".into();
        let mut c = brut("nac", "Nac's Bedroom", "Storm", 1);
        c.script_name = "Obstacle.Script.txt".into();
        let liste = convertir(vec![a, b, c]);

        let sans = comptes_par_mode(&liste, "");
        assert_eq!(
            sans,
            vec![
                ModeCompte {
                    mode: "Obstacle".into(),
                    compte: 2
                },
                ModeCompte {
                    mode: "Elite".into(),
                    compte: 1
                },
            ]
        );

        let avec = comptes_par_mode(&liste, "atria");
        // A egalite de compte, l'ordre est alphabetique : il faut bien un
        // depart, et un ordre stable vaut mieux qu'un ordre d'arrivee qui
        // changerait au gre des rafraichissements.
        assert_eq!(
            avec,
            vec![
                ModeCompte {
                    mode: "Elite".into(),
                    compte: 1
                },
                ModeCompte {
                    mode: "Obstacle".into(),
                    compte: 1
                },
            ]
        );
    }

    #[test]
    fn le_tirage_evite_les_derniers_proposes() {
        // Cinq candidats, trois deja vus : le tirage doit tomber sur l'un
        // des deux restants, quel que soit le nombre fourni.
        for alea in 0..20 {
            let choisi = tirage(5, &[0, 1, 2], alea).unwrap();
            assert!(choisi == 3 || choisi == 4, "alea {alea} a rendu {choisi}");
        }
    }

    #[test]
    fn le_tirage_repart_sur_tout_quand_tout_a_ete_propose() {
        // Mieux vaut reproposer que ne rien rendre.
        let choisi = tirage(3, &[0, 1, 2], 7).unwrap();
        assert!(choisi < 3);
    }

    #[test]
    fn le_tirage_sans_candidat_ne_rend_rien() {
        assert_eq!(tirage(0, &[], 0), None);
        assert_eq!(tirage(0, &[0], 3), None);
    }

    #[test]
    fn le_tirage_couvre_tous_les_candidats() {
        // Aucun candidat ne doit etre inatteignable a cause d'un modulo
        // mal pose.
        let vus: std::collections::HashSet<usize> =
            (0..40u64).filter_map(|a| tirage(4, &[], a)).collect();
        assert_eq!(vus.len(), 4);
    }

    #[test]
    fn la_vue_rend_tout_ce_dont_l_ecran_a_besoin() {
        let mut a = brut("atria_obs", "Atria - Obstacle", "Storm", 7);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("atria_vide", "Atria - Training", "Storm", 0);
        b.script_name = "Obstacle.Script.txt".into();
        let mut c = brut("nac_elite", "Nac's Elite", "Storm", 2);
        c.script_name = "SM_Elite_Online.Script.txt".into();
        let liste = convertir(vec![a, b, c]);

        let v = vue(&liste, &Filtre::default(), Tri::Joueurs, false);
        assert_eq!(v.total, 3);
        assert_eq!(v.retenus, 3);
        assert_eq!(
            v.peuples
                .iter()
                .map(|s| s.login.as_str())
                .collect::<Vec<_>>(),
            vec!["atria_obs", "nac_elite"]
        );
        assert_eq!(
            v.vides.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
            vec!["atria_vide"]
        );
        assert_eq!(v.comptes.len(), 2);
    }

    #[test]
    fn la_vue_dit_combien_les_filtres_ecartent() {
        let liste = convertir(vec![
            brut("a", "Atria", "Storm", 3),
            brut("b", "Nac", "Storm", 1),
            brut("c", "Yox", "Storm", 0),
        ]);
        let v = vue(
            &liste,
            &Filtre {
                recherche: "atria",
                ..Filtre::default()
            },
            Tri::Joueurs,
            false,
        );
        // total ne bouge pas, retenus dit ce qui reste : c'est cet ecart
        // qui explique une liste courte au lieu de la laisser inexpliquee.
        assert_eq!(v.total, 3);
        assert_eq!(v.retenus, 1);
        // Et les comptes par mode restent ceux de la recherche, pour que
        // les puces disent ce qu'elles rapporteraient.
        assert_eq!(v.comptes.iter().map(|c| c.compte).sum::<usize>(), 1);
    }

    #[test]
    fn une_vue_sans_aucun_serveur_ne_panique_pas() {
        let v = vue(&[], &Filtre::default(), Tri::Nom, true);
        assert_eq!(v.total, 0);
        assert_eq!(v.retenus, 0);
        assert!(v.peuples.is_empty());
        assert!(v.vides.is_empty());
        assert!(v.comptes.is_empty());
    }

    #[test]
    fn la_recherche_porte_aussi_sur_le_mode() {
        // Avec une centaine de serveurs et autant de modes
        // communautaires, taper « obstacle » doit ramener les serveurs
        // d'Obstacle sans avoir a trouver la bonne puce.
        let mut a = brut("a", "Atria - Training", "Storm", 3);
        a.script_name = "Obstacle.Script.txt".into();
        let mut b = brut("b", "Atria - Duel", "Storm", 2);
        b.script_name = "SM_Elite_Online.Script.txt".into();
        let liste = convertir(vec![a, b]);

        let trouve = filtrer(
            &liste,
            &Filtre {
                recherche: "obstacle",
                ..Filtre::default()
            },
        );
        assert_eq!(trouve.len(), 1);
        assert_eq!(trouve[0].login, "a");
    }

    #[test]
    fn le_filtre_du_mot_de_passe_va_dans_les_deux_sens() {
        let mut prive = brut("prive", "Clan", "Storm", 1);
        prive.is_private = true;
        let liste = convertir(vec![prive, brut("ouvert", "Public", "Storm", 4)]);

        let tout = filtrer(&liste, &Filtre::default());
        assert_eq!(tout.len(), 2);

        let sans = filtrer(
            &liste,
            &Filtre {
                prive: Some(false),
                ..Filtre::default()
            },
        );
        assert_eq!(sans.len(), 1);
        assert_eq!(sans[0].login, "ouvert");

        let seulement = filtrer(
            &liste,
            &Filtre {
                prive: Some(true),
                ..Filtre::default()
            },
        );
        assert_eq!(seulement.len(), 1);
        assert_eq!(seulement[0].login, "prive");
    }

    #[test]
    fn le_filtre_des_favoris_ne_garde_que_les_logins_retenus() {
        let liste = convertir(vec![
            brut("a", "A", "Storm", 3),
            brut("b", "B", "Storm", 2),
            brut("c", "C", "Storm", 1),
        ]);
        let favoris = vec!["a".to_string(), "c".to_string()];
        let gardes = filtrer(
            &liste,
            &Filtre {
                favoris_seuls: true,
                favoris: &favoris,
                ..Filtre::default()
            },
        );
        assert_eq!(
            gardes.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
            vec!["a", "c"]
        );

        // Un favori qui n'est plus en ligne ne fait pas apparaitre de
        // ligne fantome, il disparait simplement du resultat.
        let disparus = vec!["z".to_string()];
        let rien = filtrer(
            &liste,
            &Filtre {
                favoris_seuls: true,
                favoris: &disparus,
                ..Filtre::default()
            },
        );
        assert!(rien.is_empty());
    }

    #[test]
    fn le_sens_du_tri_s_inverse_sur_les_trois_criteres() {
        let liste = convertir(vec![
            brut("a", "Alpha", "Storm", 1),
            brut("b", "Bravo", "Storm", 9),
            brut("c", "Charlie", "Storm", 5),
        ]);

        for (tri, attendu_naturel) in [
            (Tri::Joueurs, vec!["b", "c", "a"]),
            (Tri::Nom, vec!["a", "b", "c"]),
        ] {
            let mut naturel = liste.clone();
            trier(&mut naturel, tri, ordre_naturel_croissant(tri));
            assert_eq!(
                naturel.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
                attendu_naturel,
                "{tri:?} dans son ordre naturel"
            );

            let mut inverse = liste.clone();
            trier(&mut inverse, tri, !ordre_naturel_croissant(tri));
            let mut attendu_inverse = attendu_naturel.clone();
            attendu_inverse.reverse();
            assert_eq!(
                inverse.iter().map(|s| s.login.as_str()).collect::<Vec<_>>(),
                attendu_inverse,
                "{tri:?} inverse"
            );
        }
    }

    #[test]
    fn l_ordre_naturel_depend_du_critere() {
        // La question qu'on se pose en ouvrant la liste est « ou y a-t-il
        // du monde », donc la frequentation part du plus peuple. Un nom
        // part de A.
        assert!(!ordre_naturel_croissant(Tri::Joueurs));
        assert!(ordre_naturel_croissant(Tri::Nom));
        assert!(ordre_naturel_croissant(Tri::Mode));
    }

    #[test]
    fn les_comptes_vont_du_mode_le_plus_joue_au_moins_joue() {
        let faire = |login: &str, script: &str, joueurs: u32| {
            let mut s = brut(login, login, "Storm", joueurs);
            s.script_name = script.to_string();
            s
        };
        let liste = convertir(vec![
            faire("a", "SM_Elite_Online.Script.txt", 1),
            faire("b", "Obstacle.Script.txt", 1),
            faire("c", "Obstacle.Script.txt", 1),
            faire("d", "Obstacle.Script.txt", 1),
            faire("e", "SM_Royal_Online.Script.txt", 1),
            faire("f", "SM_Elite_Online.Script.txt", 1),
        ]);
        let comptes = comptes_par_mode(&liste, "");
        assert_eq!(
            comptes
                .iter()
                .map(|c| (c.mode.as_str(), c.compte))
                .collect::<Vec<_>>(),
            vec![("Obstacle", 3), ("Elite", 2), ("Royal", 1)]
        );
    }

    #[test]
    fn un_critere_neuf_part_dans_son_sens_naturel() {
        assert_eq!(tri_suivant(Tri::Joueurs, false, Tri::Nom), (Tri::Nom, true));
        assert_eq!(tri_suivant(Tri::Nom, true, Tri::Mode), (Tri::Mode, true));
        assert_eq!(
            tri_suivant(Tri::Nom, true, Tri::Joueurs),
            (Tri::Joueurs, false)
        );
    }

    #[test]
    fn le_deuxieme_clic_renverse_le_sens() {
        assert_eq!(tri_suivant(Tri::Nom, true, Tri::Nom), (Tri::Nom, false));
        assert_eq!(
            tri_suivant(Tri::Joueurs, false, Tri::Joueurs),
            (Tri::Joueurs, true)
        );
    }

    #[test]
    fn le_troisieme_clic_ramene_au_tri_par_defaut() {
        // Nom, A a Z, puis Z a A, puis neutre.
        let (tri, croissant) = tri_suivant(Tri::Nom, false, Tri::Nom);
        assert_eq!(tri, Tri::default());
        assert_eq!(croissant, ordre_naturel_croissant(Tri::default()));
        // Et pour la frequentation, qui EST le tri par defaut, le cycle
        // se referme sur elle meme : decroissant, croissant, decroissant.
        assert_eq!(
            tri_suivant(Tri::Joueurs, true, Tri::Joueurs),
            (Tri::Joueurs, false)
        );
    }

    #[test]
    fn le_cycle_complet_revient_a_son_point_de_depart() {
        let depart = (Tri::default(), ordre_naturel_croissant(Tri::default()));
        let mut etat = depart;
        for _ in 0..3 {
            etat = tri_suivant(etat.0, etat.1, Tri::Mode);
        }
        assert_eq!(etat, depart);
    }

    #[test]
    fn un_nom_entierement_code_retombe_sur_le_login_des_deux_cotes() {
        let mut brut = ServeurApi {
            login: "atria_obstacle".to_string(),
            name: "$fff$o$i$n".to_string(),
            environment: ENVIRONNEMENT.to_string(),
            ..Default::default()
        };
        brut.script_name = "Obstacle".to_string();
        let liste = convertir(vec![brut]);
        assert_eq!(liste[0].nom, "atria_obstacle");
        assert_eq!(liste[0].nom_segments.len(), 1);
        assert_eq!(liste[0].nom_segments[0].texte, "atria_obstacle");
        assert_eq!(liste[0].nom_segments[0].couleur, None);
    }

    #[test]
    fn le_nom_affiche_et_le_nom_nu_disent_la_meme_chose() {
        let mut brut = ServeurApi {
            login: "nac".to_string(),
            name: "$o$nNac's $i$63EBedroom".to_string(),
            environment: ENVIRONNEMENT.to_string(),
            ..Default::default()
        };
        brut.script_name = "Obstacle".to_string();
        let liste = convertir(vec![brut]);
        let colle: String = liste[0]
            .nom_segments
            .iter()
            .map(|s| s.texte.as_str())
            .collect();
        assert_eq!(colle, liste[0].nom);
        // Et la couleur du second morceau est remontee, pas recopiee.
        assert!(liste[0].nom_segments[1].couleur.is_some());
    }
}
