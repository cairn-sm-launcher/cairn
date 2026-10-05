//! Le rang des joueurs, ses regles et son bareme.
//!
//! Tout ce module est **pur**. Il ne lit rien, il n'ecrit rien, il ne
//! sait pas ce qu'est un serveur. On lui donne un joueur, un resultat de
//! match, et il rend le nouveau rang avec le detail du calcul. C'est ce
//! qui permet de l'ecrire et de le verifier aujourd'hui, alors que ni le
//! hub ni le mode de jeu n'existent : le jour ou les vrais resultats
//! arrivent, le calcul est deja juste.
//!
//! # Ce que ce systeme est, en realite
//!
//! Il vaut la peine de le nommer. Les regles [`attendu`] et
//! [`bataille_difficile`] sont le terme d'esperance de score d'un Elo,
//! exprime en paliers au lieu d'une courbe continue. Flow a reconstruit a
//! la main, par intuition de ce qui est juste, ce qu'un Elo calcule par
//! formule.
//!
//! Consequence pratique, et elle simplifie beaucoup : **un seul nombre
//! suffit.** Pas besoin d'un classement cache derriere les points
//! visibles comme le font les gros jeux. Les points de rang sont la note,
//! et l'appariement s'equilibre dessus.
//!
//! # Le modele de calcul
//!
//! Chaque regle rend un **pourcentage applique a l'ampleur** du
//! changement de points. Positif veut dire « il se passe plus de
//! choses » : plus de points gagnes sur une victoire, plus de points
//! perdus sur une defaite. Negatif veut dire l'inverse.
//!
//! Les pourcentages **s'additionnent**, puis s'appliquent une seule fois
//! a la base. C'est un choix, et il se defend : le joueur peut lire
//! « plus 10 et plus 15, donc plus 25 % », ce qu'une multiplication en
//! cascade ne permet pas. C'est aussi beaucoup plus simple a tester.
//!
//! Deux bornes encadrent le resultat. Le total est **plancher a -100 %**,
//! sans quoi une defaite tres protegee rapporterait des points, ce qui
//! n'a aucun sens. Et l'ampleur finale est **plafonnee a
//! [`PLAFOND_PAR_MATCH`]**.
//!
//! # Pourquoi un plafond
//!
//! Ce n'est pas une securite, c'est ce qui donne son sens a la note. Un
//! rang doit se gagner en plusieurs matchs et se perdre en plusieurs
//! matchs. Si une seule partie deplace le joueur de la moitie d'un
//! palier, le rang cesse de decrire son niveau et se met a decrire sa
//! derniere soiree.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------
//  Le bareme
// ---------------------------------------------------------------------
//
//  Ces cinq nombres sont les seuls a regler. Ils sont groupes ici pour
//  qu'on puisse les bouger a l'usage sans aller chercher dans le code.

/// Les points qu'il faut traverser pour changer de palier.
///
/// Cent, parce que c'est le plus lisible : un pourcentage se lit alors
/// directement en points.
pub const POINTS_PAR_PALIER: u32 = 100;

/// Ce que rapporte une victoire avant toute regle.
///
/// Vingt-deux, soit environ cinq victoires propres pour monter d'un
/// palier.
pub const GAIN_BASE: i32 = 22;

/// Ce que coute une defaite avant toute regle.
///
/// Legerement moins que le gain : la progression est un peu plus facile
/// que la chute, ce qui evite qu'une mauvaise soiree efface une semaine.
pub const PERTE_BASE: i32 = 20;

/// Le maximum de points qu'un seul match peut deplacer.
///
/// La moitie d'un palier. Avec le bareme actuel il ne mord presque
/// jamais, et c'est le signe qu'il est bien place : il est la pour les
/// cumuls extremes, pas pour brider le jeu ordinaire.
pub const PLAFOND_PAR_MATCH: i32 = 50;

/// Le plus haut palier qu'on puisse atteindre aux points seuls.
///
/// Diamant ne s'atteint pas en accumulant : c'est le top 10, donc une
/// place dans un classement et non un total. Le hub l'attribue, ce
/// module se contente de monter jusqu'a Platine. Un joueur deja Diamant
/// qui redescend repasse Platine normalement.
pub const PALIER_MAX_AUX_POINTS: u8 = 3;

// ---------------------------------------------------------------------
//  Les paliers
// ---------------------------------------------------------------------

/// Les cinq paliers, du plus bas au plus haut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Palier {
    Bronze,
    Argent,
    Or,
    Platine,
    Diamant,
}

impl Palier {
    /// Le rang du palier, de 0 pour Bronze a 4 pour Diamant.
    pub fn indice(self) -> u8 {
        match self {
            Palier::Bronze => 0,
            Palier::Argent => 1,
            Palier::Or => 2,
            Palier::Platine => 3,
            Palier::Diamant => 4,
        }
    }

    /// Le palier a cet indice, borne aux deux bouts.
    pub fn depuis_indice(indice: u8) -> Self {
        match indice {
            0 => Palier::Bronze,
            1 => Palier::Argent,
            2 => Palier::Or,
            3 => Palier::Platine,
            _ => Palier::Diamant,
        }
    }
}

/// Le rang d'un joueur : un palier, et sa progression dedans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rang {
    pub palier: Palier,
    /// De 0 a [`POINTS_PAR_PALIER`] moins un.
    pub points: u32,
}

impl Rang {
    pub fn nouveau(palier: Palier, points: u32) -> Self {
        Self {
            palier,
            points: points.min(POINTS_PAR_PALIER - 1),
        }
    }

    /// Le rang en un seul nombre, de 0 a 499.
    ///
    /// C'est cette echelle continue qui sert a comparer deux joueurs et
    /// a equilibrer deux equipes. Un Bronze a 95 points est presque un
    /// Argent, et un simple indice de palier le perdrait.
    pub fn score(self) -> u32 {
        self.palier.indice() as u32 * POINTS_PAR_PALIER + self.points
    }

    /// Le rang correspondant a un score continu.
    pub fn depuis_score(score: u32) -> Self {
        let indice = (score / POINTS_PAR_PALIER).min(Palier::Diamant.indice() as u32) as u8;
        Self::nouveau(
            Palier::depuis_indice(indice),
            score - indice as u32 * POINTS_PAR_PALIER,
        )
    }
}

impl Default for Rang {
    fn default() -> Self {
        Self {
            palier: Palier::Bronze,
            points: 0,
        }
    }
}

// ---------------------------------------------------------------------
//  Ce qu'on donne au calcul
// ---------------------------------------------------------------------

/// Le joueur, tel que le hub le connait avant le match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Joueur {
    pub rang: Rang,
    /// Les resultats consecutifs, ce match compris.
    ///
    /// Trois victoires de suite donnent 3, deux defaites de suite
    /// donnent 2. Le sens est porte par le resultat du match, pas par le
    /// signe : une serie ne melange jamais les deux.
    pub consecutifs: u32,
    /// Le joueur est-il deja monte au dessus de Bronze un jour.
    ///
    /// C'est ce qui separe un debutant, protege, d'un joueur retrograde
    /// en Bronze, qui ne l'est pas.
    pub deja_monte: bool,
    /// Ce match est-il le premier apres une montee de palier.
    pub vient_de_monter: bool,
}

/// Le match, vu du cote du joueur dont on calcule le rang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Affrontement {
    pub gagne: bool,
    /// Le palier moyen de son equipe, arrondi.
    pub mon_palier: u8,
    /// Celui de l'equipe d'en face.
    pub palier_adverse: u8,
}

impl Affrontement {
    /// L'ecart de palier entre les deux equipes, toujours positif.
    pub fn ecart(&self) -> u8 {
        self.mon_palier.abs_diff(self.palier_adverse)
    }

    /// Mon equipe partait-elle favorite.
    pub fn favori(&self) -> bool {
        self.mon_palier > self.palier_adverse
    }
}

// ---------------------------------------------------------------------
//  Les regles, une par fonction
// ---------------------------------------------------------------------

/// Les regles du bareme, pour que l'interface sache quoi afficher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regle {
    SerieVictoires,
    SerieDefaites,
    Etalonnage,
    BatailleDifficile,
    Renversement,
    Volatile,
    Consolation,
    Attendu,
}

/// Une regle qui s'est appliquee, et de combien.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effet {
    pub regle: Regle,
    /// En pourcentage de l'ampleur de base, positif ou negatif.
    pub pourcentage: i32,
}

/// La serie en cours, victoires comme defaites.
///
/// La premiere ne donne rien, la deuxieme 5 %, la troisieme 10 %, et
/// ainsi de suite jusqu'a 20 %. Le signe est toujours positif : sur une
/// victoire ca augmente le gain, sur une defaite ca augmente la perte.
pub fn serie(consecutifs: u32) -> i32 {
    if consecutifs < 2 {
        return 0;
    }
    (((consecutifs - 1) * 5) as i32).min(20)
}

/// L'ecart de niveau entre les deux equipes, en continu.
///
/// Cinq pour cent par palier d'ecart, jusqu'a vingt. C'est le terme
/// general, celui qui s'applique des qu'il y a un desequilibre, la ou
/// les regles nommees plus bas ne se declenchent que dans des situations
/// precises.
///
/// Le sens est celui qu'on attend : l'equipe la plus faible gagne plus
/// et perd moins, la plus forte l'inverse.
pub fn etalonnage(a: &Affrontement) -> i32 {
    let ampleur = ((a.ecart() as i32) * 5).min(20);
    if ampleur == 0 {
        return 0;
    }
    // Favori qui gagne : moins. Favori qui perd : plus. Outsider qui
    // gagne : plus. Outsider qui perd : moins.
    if a.favori() == a.gagne {
        -ampleur
    } else {
        ampleur
    }
}

/// La victoire de l'outsider, a plus d'un palier d'ecart.
///
/// Dix pour cent de plus. Les exemples de Flow sont Bronze contre Or et
/// Argent contre Platine, donc bien deux paliers et non un.
pub fn bataille_difficile(a: &Affrontement) -> i32 {
    if a.gagne && !a.favori() && a.ecart() >= 2 {
        10
    } else {
        0
    }
}

/// La defaite du favori, a plus d'un palier d'ecart.
pub fn renversement(a: &Affrontement) -> i32 {
    if !a.gagne && a.favori() && a.ecart() >= 2 {
        5
    } else {
        0
    }
}

/// La defaite qui suit de peu une montee de palier.
///
/// Cinq pour cent de perte en plus. Elle ne s'applique qu'a un match
/// desequilibre, ce que Flow appelle un match d'etalonnage : monter d'un
/// palier puis perdre contre des joueurs de son propre niveau n'a rien
/// d'anormal et ne merite pas de punition.
pub fn volatile(a: &Affrontement, vient_de_monter: bool) -> i32 {
    if !a.gagne && vient_de_monter && a.ecart() >= 1 {
        5
    } else {
        0
    }
}

/// La defaite de l'outsider, tres attenuee.
///
/// La courbe est celle que Flow a ecrite, et elle n'est pas lineaire :
/// un quart de la perte effacee a deux paliers d'ecart, trois quarts a
/// trois, la totalite a quatre. Perdre contre beaucoup plus fort que soi
/// ne doit rien couter.
pub fn consolation(a: &Affrontement) -> i32 {
    if a.gagne || a.favori() {
        return 0;
    }
    match a.ecart() {
        0 | 1 => 0,
        2 => -25,
        3 => -75,
        _ => -100,
    }
}

/// La victoire du favori, peu recompensee.
///
/// Trente pour cent de moins par palier d'ecart, quarante quand le
/// vainqueur est Diamant. Le taux plus dur en haut est voulu : Diamant
/// est le top dix, battre plus faible que soi n'y apprend rien.
///
/// Le calcul reproduit exactement la table de Flow et la prolonge aux
/// paliers qu'elle ne nommait pas : Platine contre Or vaut moins 30,
/// contre Argent moins 60, contre Bronze moins 90, et Diamant contre
/// Platine moins 40, contre Or moins 80, plus bas moins 100.
pub fn attendu(a: &Affrontement) -> i32 {
    if !a.gagne || !a.favori() {
        return 0;
    }
    let taux = if a.mon_palier >= Palier::Diamant.indice() {
        40
    } else {
        30
    };
    (-(a.ecart() as i32) * taux).max(-100)
}

// ---------------------------------------------------------------------
//  Le calcul complet
// ---------------------------------------------------------------------

/// Ce qui a empeche le joueur de perdre des points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protection {
    /// Un Bronze qui n'est jamais monte ne perd rien.
    Debutant,
    /// La defaite aurait fait descendre d'un palier, elle laisse a zero.
    Retrogradation,
}

/// Le resultat complet d'un match pour un joueur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resultat {
    pub rang_avant: Rang,
    pub rang_apres: Rang,
    /// Les points gagnes ou perdus, deja plafonnes.
    pub delta: i32,
    /// Le detail du calcul, pour que l'interface puisse l'expliquer.
    pub effets: Vec<Effet>,
    pub promotion: bool,
    pub retrogradation: bool,
    pub protection: Option<Protection>,
}

/// Rassemble les regles qui s'appliquent a ce match.
///
/// Rendue a part pour que l'interface puisse montrer le detail avant
/// meme de connaitre le nouveau rang, et pour que chaque regle reste
/// verifiable isolement.
pub fn effets(joueur: &Joueur, a: &Affrontement) -> Vec<Effet> {
    // **L'etalonnage s'efface devant l'attendu.** Les deux reduisent le
    // gain du favori et les deux le font proportionnellement a l'ecart,
    // donc les cumuler compte deux fois la meme idee. Concretement, la
    // table de Flow dit que Platine battant Bronze rapporte 10 % du gain
    // de base ; l'etalonnage ajoutait -15 %, le total passait sous
    // -100 % et le resultat tombait a zero, ce qui effacait en silence
    // un chiffre qu'il avait ecrit explicitement.
    //
    // Decision de Flow le 2026-09-30. Elle ne vaut que pour l'attendu :
    // la consolation, qui est son symetrique sur les defaites, continue
    // de se cumuler avec l'etalonnage, et la question de faire pareil
    // des deux cotes reste posee.
    let attendu = attendu(a);
    let etalonnage = if attendu != 0 { 0 } else { etalonnage(a) };

    let candidats = [
        (
            if a.gagne {
                Regle::SerieVictoires
            } else {
                Regle::SerieDefaites
            },
            serie(joueur.consecutifs),
        ),
        (Regle::Etalonnage, etalonnage),
        (Regle::BatailleDifficile, bataille_difficile(a)),
        (Regle::Renversement, renversement(a)),
        (Regle::Volatile, volatile(a, joueur.vient_de_monter)),
        (Regle::Consolation, consolation(a)),
        (Regle::Attendu, attendu),
    ];
    candidats
        .into_iter()
        .filter(|(_, pourcentage)| *pourcentage != 0)
        .map(|(regle, pourcentage)| Effet { regle, pourcentage })
        .collect()
}

/// Calcule le nouveau rang d'un joueur apres un match.
pub fn calculer(joueur: &Joueur, a: &Affrontement) -> Resultat {
    let effets = effets(joueur, a);

    // Le debutant ne perd rien, et la question s'arrete la.
    if !a.gagne && joueur.rang.palier == Palier::Bronze && !joueur.deja_monte {
        return Resultat {
            rang_avant: joueur.rang,
            rang_apres: joueur.rang,
            delta: 0,
            effets,
            promotion: false,
            retrogradation: false,
            protection: Some(Protection::Debutant),
        };
    }

    // Les pourcentages s'additionnent, puis s'appliquent une fois. Le
    // plancher a -100 % empeche une defaite de rapporter des points.
    let total: i32 = effets.iter().map(|e| e.pourcentage).sum::<i32>().max(-100);
    let base = if a.gagne { GAIN_BASE } else { PERTE_BASE };
    // Arrondi au plus proche, et non troncature : sans lui, une regle a
    // -90 % sur une base de 22 rendrait 2 points la ou 2,2 en vaut 2, ce
    // qui se voit peu, mais une base impaire ferait disparaitre un point
    // a chaque match.
    let ampleur = ((base as f64) * (100 + total) as f64 / 100.0).round() as i32;
    let ampleur = ampleur.clamp(0, PLAFOND_PAR_MATCH);

    let delta = if a.gagne { ampleur } else { -ampleur };
    let (rang_apres, protection) = appliquer(joueur.rang, delta);

    Resultat {
        rang_avant: joueur.rang,
        rang_apres,
        delta: if a.gagne {
            delta
        } else {
            // Une perte annulee par la protection n'est pas une perte.
            rang_apres.score() as i32 - joueur.rang.score() as i32
        },
        effets,
        promotion: rang_apres.palier > joueur.rang.palier,
        retrogradation: rang_apres.palier < joueur.rang.palier,
        protection,
    }
}

/// Pose un changement de points sur un rang, avec ses deux bornes.
///
/// **La protection de retrogradation vit ici, et elle ne demande aucun
/// drapeau.** Une defaite qui ferait passer sous zero laisse a zero, et
/// c'est le fait d'etre deja a zero qui autorise la descente au coup
/// suivant. L'etat est donc porte par les points eux memes, ce qui evite
/// un champ de plus a tenir a jour et a synchroniser.
///
/// Vers le haut, la montee s'arrete a [`PALIER_MAX_AUX_POINTS`] : Diamant
/// se gagne au classement, pas au total.
pub fn appliquer(rang: Rang, delta: i32) -> (Rang, Option<Protection>) {
    let score = rang.score() as i32;
    let vise = score + delta;

    if delta < 0 {
        // Deja au plancher de son palier : la descente est permise.
        if rang.points == 0 {
            let plancher = 0;
            return (Rang::depuis_score(vise.max(plancher) as u32), None);
        }
        // Sinon on ne descend pas de palier ce coup-ci.
        let plancher = rang.palier.indice() as i32 * POINTS_PAR_PALIER as i32;
        if vise < plancher {
            return (
                Rang::nouveau(rang.palier, 0),
                Some(Protection::Retrogradation),
            );
        }
        return (Rang::depuis_score(vise as u32), None);
    }

    // Le plafond depend de la ou l'on est deja. Un joueur sous Diamant
    // s'arrete en haut de Platine, parce que Diamant se gagne au
    // classement. Mais un joueur DEJA Diamant continue de monter dans son
    // palier : ses points servent a le departager des neuf autres, donc
    // les bloquer reviendrait a figer le haut du classement.
    let plafond = if rang.palier == Palier::Diamant {
        (Palier::Diamant.indice() as i32 + 1) * POINTS_PAR_PALIER as i32 - 1
    } else {
        (PALIER_MAX_AUX_POINTS as i32 + 1) * POINTS_PAR_PALIER as i32 - 1
    };
    (Rang::depuis_score(vise.min(plafond) as u32), None)
}

// ---------------------------------------------------------------------
//  Le palier moyen d'une equipe
// ---------------------------------------------------------------------

/// Le palier moyen d'une equipe, arrondi au plus proche.
///
/// La moyenne se fait sur le **score continu** et non sur l'indice de
/// palier : une equipe de trois Bronze a 90 points est bien plus proche
/// d'Argent que trois Bronze a 5 points, et une moyenne d'indices les
/// confondrait.
///
/// Une equipe vide rend Bronze, faute de mieux.
pub fn palier_moyen(equipe: &[Rang]) -> u8 {
    if equipe.is_empty() {
        return 0;
    }
    let somme: u32 = equipe.iter().map(|r| r.score()).sum();
    let moyenne = somme as f64 / equipe.len() as f64;
    (moyenne / POINTS_PAR_PALIER as f64).round() as u8
}

// ---------------------------------------------------------------------
//  L'equilibrage des equipes
// ---------------------------------------------------------------------

/// Le nombre de joueurs au dela duquel on ne cherche plus la perfection.
///
/// La recherche exhaustive coute deux puissance N. A seize joueurs c'est
/// soixante-cinq mille combinaisons, soit quelques millisecondes, et
/// c'est deja bien au dela des six d'un match d'Elite. Au dela on
/// refuse plutot que de faire attendre.
pub const JOUEURS_MAX_EQUILIBRAGE: usize = 16;

/// Deux equipes, et ce qui les separe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equilibrage {
    /// Les indices des joueurs, dans la liste donnee.
    pub equipe_a: Vec<usize>,
    pub equipe_b: Vec<usize>,
    /// La difference de score total entre les deux, toujours positive.
    pub ecart: u32,
}

/// Repartit des joueurs en deux equipes aussi proches que possible.
///
/// **Exhaustif, donc optimal.** Pour six joueurs il y a dix repartitions
/// distinctes, on les essaie toutes. Une heuristique irait plus vite et
/// se tromperait parfois, ce qui serait absurde a cette taille.
///
/// Le resultat est **deterministe** : a egalite de qualite, c'est la
/// premiere combinaison dans l'ordre naturel qui gagne. Deux appels sur
/// la meme liste rendent donc la meme chose, ce qui compte autant pour
/// les tests que pour le joueur, qui ne doit pas voir les equipes
/// changer parce qu'on a rafraichi l'ecran.
///
/// Rend `None` quand le nombre de joueurs est impair, nul, ou trop grand.
pub fn equilibrer(joueurs: &[Rang]) -> Option<Equilibrage> {
    let n = joueurs.len();
    if n == 0 || n % 2 != 0 || n > JOUEURS_MAX_EQUILIBRAGE {
        return None;
    }
    let moitie = n / 2;
    let scores: Vec<i64> = joueurs.iter().map(|r| r.score() as i64).collect();
    let total: i64 = scores.iter().sum();

    let mut meilleur: Option<(i64, Vec<usize>)> = None;
    // On fixe le joueur 0 dans l'equipe A, ce qui supprime la moitie des
    // combinaisons, celles qui ne font qu'echanger les deux equipes.
    for masque in 0u32..(1u32 << n) {
        if masque & 1 == 0 || masque.count_ones() as usize != moitie {
            continue;
        }
        let a: Vec<usize> = (0..n).filter(|i| masque >> i & 1 == 1).collect();
        let somme_a: i64 = a.iter().map(|&i| scores[i]).sum();
        let ecart = (somme_a * 2 - total).abs();
        // `is_none_or` n'existe que depuis Rust 1.82 et la caisse tient
        // une version minimale plus basse, voir le Cargo.toml.
        if meilleur.as_ref().map_or(true, |(e, _)| ecart < *e) {
            meilleur = Some((ecart, a));
        }
    }

    let (ecart, equipe_a) = meilleur?;
    let equipe_b: Vec<usize> = (0..n).filter(|i| !equipe_a.contains(i)).collect();
    Some(Equilibrage {
        equipe_a,
        equipe_b,
        ecart: ecart as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rang(palier: u8, points: u32) -> Rang {
        Rang::nouveau(Palier::depuis_indice(palier), points)
    }

    fn joueur(palier: u8, points: u32) -> Joueur {
        Joueur {
            rang: rang(palier, points),
            consecutifs: 1,
            deja_monte: palier > 0,
            vient_de_monter: false,
        }
    }

    fn duel(gagne: bool, mien: u8, adverse: u8) -> Affrontement {
        Affrontement {
            gagne,
            mon_palier: mien,
            palier_adverse: adverse,
        }
    }

    // -----------------------------------------------------------------
    //  Les paliers et le score continu
    // -----------------------------------------------------------------

    #[test]
    fn le_score_et_le_rang_font_l_aller_et_le_retour() {
        for palier in 0..=4u8 {
            for points in [0, 1, 50, 99] {
                let r = rang(palier, points);
                assert_eq!(Rang::depuis_score(r.score()), r, "{palier}/{points}");
            }
        }
    }

    #[test]
    fn le_score_ordonne_les_joueurs_comme_l_intuition() {
        // Un Bronze presque Argent vaut plus qu'un Bronze qui debute, et
        // moins que le premier Argent. C'est tout l'interet de l'echelle
        // continue face a un simple indice de palier.
        assert!(rang(0, 95).score() > rang(0, 5).score());
        assert!(rang(0, 99).score() < rang(1, 0).score());
        assert!(rang(3, 99).score() < rang(4, 0).score());
    }

    // -----------------------------------------------------------------
    //  Les regles, une par une
    // -----------------------------------------------------------------

    #[test]
    fn la_serie_demarre_a_la_deuxieme_et_plafonne_a_vingt() {
        // « apres une premiere victoire, la deuxieme recompense de 5 %,
        // troisieme = 10 %, jusqu'a un maximum de 20 % ».
        assert_eq!(serie(0), 0);
        assert_eq!(serie(1), 0);
        assert_eq!(serie(2), 5);
        assert_eq!(serie(3), 10);
        assert_eq!(serie(4), 15);
        assert_eq!(serie(5), 20);
        assert_eq!(serie(12), 20);
    }

    #[test]
    fn l_etalonnage_avantage_le_plus_faible_dans_les_deux_sens() {
        // Outsider : il gagne plus et perd moins.
        assert_eq!(etalonnage(&duel(true, 0, 2)), 10);
        assert_eq!(etalonnage(&duel(false, 0, 2)), -10);
        // Favori : l'inverse.
        assert_eq!(etalonnage(&duel(true, 2, 0)), -10);
        assert_eq!(etalonnage(&duel(false, 2, 0)), 10);
    }

    #[test]
    fn l_etalonnage_vaut_cinq_par_palier_et_plafonne_a_vingt() {
        assert_eq!(etalonnage(&duel(true, 0, 0)), 0);
        assert_eq!(etalonnage(&duel(true, 0, 1)), 5);
        assert_eq!(etalonnage(&duel(true, 0, 2)), 10);
        assert_eq!(etalonnage(&duel(true, 0, 3)), 15);
        assert_eq!(etalonnage(&duel(true, 0, 4)), 20);
    }

    #[test]
    fn la_bataille_difficile_demande_deux_paliers_et_une_victoire() {
        // Les exemples de Flow : Bronze contre Or, Argent contre Platine.
        assert_eq!(bataille_difficile(&duel(true, 0, 2)), 10);
        assert_eq!(bataille_difficile(&duel(true, 1, 3)), 10);
        // Un seul palier d'ecart ne suffit pas, « plus d'un tier ».
        assert_eq!(bataille_difficile(&duel(true, 0, 1)), 0);
        // Et il faut avoir gagne, et ne pas avoir ete favori.
        assert_eq!(bataille_difficile(&duel(false, 0, 2)), 0);
        assert_eq!(bataille_difficile(&duel(true, 2, 0)), 0);
    }

    #[test]
    fn le_renversement_est_la_defaite_du_favori() {
        assert_eq!(renversement(&duel(false, 2, 0)), 5);
        assert_eq!(renversement(&duel(false, 1, 0)), 0); // un seul palier
        assert_eq!(renversement(&duel(true, 2, 0)), 0); // il a gagne
        assert_eq!(renversement(&duel(false, 0, 2)), 0); // il n'etait pas favori
    }

    #[test]
    fn volatile_ne_frappe_qu_apres_une_montee_et_sur_un_match_desequilibre() {
        assert_eq!(volatile(&duel(false, 1, 2), true), 5);
        // Contre son propre niveau, perdre apres une montee n'a rien
        // d'anormal : ce n'est pas un match d'etalonnage.
        assert_eq!(volatile(&duel(false, 1, 1), true), 0);
        // Sans montee recente, rien.
        assert_eq!(volatile(&duel(false, 1, 2), false), 0);
        // Sur une victoire, rien.
        assert_eq!(volatile(&duel(true, 1, 2), true), 0);
    }

    #[test]
    fn la_consolation_suit_la_courbe_de_flow() {
        // « Tier 0 Vs Tier 2 = 25 % ; Tier 0 Vs Tier 3 = 75 % ;
        //   Tier 0 vs Tier 4 = 100 % »
        assert_eq!(consolation(&duel(false, 0, 2)), -25);
        assert_eq!(consolation(&duel(false, 0, 3)), -75);
        assert_eq!(consolation(&duel(false, 0, 4)), -100);
        // Elle ne s'applique ni a un seul palier d'ecart, ni au favori,
        // ni a une victoire.
        assert_eq!(consolation(&duel(false, 0, 1)), 0);
        assert_eq!(consolation(&duel(false, 4, 0)), 0);
        assert_eq!(consolation(&duel(true, 0, 3)), 0);
    }

    #[test]
    fn attendu_reproduit_exactement_la_table_de_flow() {
        // « T3 VS T0 = 90 % de points en moins; T3 VS T1 = 60 % ;
        //   T3 VS T2 = 30 % »
        assert_eq!(attendu(&duel(true, 3, 2)), -30);
        assert_eq!(attendu(&duel(true, 3, 1)), -60);
        assert_eq!(attendu(&duel(true, 3, 0)), -90);
        // « T4 VS T0,T1 = 100 % T4 VS T2 = 80 %, T4 VS T3 = 40 % »
        assert_eq!(attendu(&duel(true, 4, 3)), -40);
        assert_eq!(attendu(&duel(true, 4, 2)), -80);
        assert_eq!(attendu(&duel(true, 4, 1)), -100);
        assert_eq!(attendu(&duel(true, 4, 0)), -100);
    }

    #[test]
    fn attendu_se_prolonge_aux_paliers_que_la_table_ne_nommait_pas() {
        // Trente pour cent par palier d'ecart, la regle derriere la table.
        assert_eq!(attendu(&duel(true, 1, 0)), -30);
        assert_eq!(attendu(&duel(true, 2, 0)), -60);
        assert_eq!(attendu(&duel(true, 2, 1)), -30);
        // A egalite, rien : on n'etait pas favori.
        assert_eq!(attendu(&duel(true, 2, 2)), 0);
        // Sur une defaite non plus.
        assert_eq!(attendu(&duel(false, 3, 0)), 0);
    }

    // -----------------------------------------------------------------
    //  Le cumul
    // -----------------------------------------------------------------

    #[test]
    fn un_match_equilibre_sans_serie_rend_la_base() {
        let r = calculer(&joueur(2, 50), &duel(true, 2, 2));
        assert!(r.effets.is_empty());
        assert_eq!(r.delta, GAIN_BASE);
        assert_eq!(r.rang_apres, rang(2, 50 + GAIN_BASE as u32));

        let p = calculer(&joueur(2, 50), &duel(false, 2, 2));
        assert_eq!(p.delta, -PERTE_BASE);
    }

    #[test]
    fn les_pourcentages_s_additionnent_avant_de_s_appliquer() {
        // Outsider de deux paliers, qui gagne, en pleine serie de trois.
        // Etalonnage +10, bataille difficile +10, serie +10, donc +30 %.
        // 22 x 1,30 = 28,6 soit 29.
        let mut j = joueur(0, 40);
        j.deja_monte = true;
        j.consecutifs = 3;
        let r = calculer(&j, &duel(true, 0, 2));
        let total: i32 = r.effets.iter().map(|e| e.pourcentage).sum();
        assert_eq!(total, 30);
        assert_eq!(r.delta, 29);
    }

    #[test]
    fn l_etalonnage_s_efface_devant_l_attendu() {
        // Decision de Flow : les deux comptaient deux fois la meme idee.
        // La table de Flow dit que Platine battant Bronze rapporte 10 %
        // du gain de base, soit 2 points. Avec l'etalonnage cumule le
        // total passait sous -100 % et tombait a zero.
        let r = calculer(&joueur(3, 50), &duel(true, 3, 0));
        assert!(!r.effets.iter().any(|e| e.regle == Regle::Etalonnage));
        assert_eq!(r.effets.iter().map(|e| e.pourcentage).sum::<i32>(), -90);
        assert_eq!(r.delta, 2);

        // Et sur une defaite, ou l'attendu ne se declenche pas,
        // l'etalonnage reste.
        let p = calculer(&joueur(3, 50), &duel(false, 3, 0));
        assert!(p.effets.iter().any(|e| e.regle == Regle::Etalonnage));
    }

    #[test]
    fn la_victoire_attendue_du_diamant_ne_rapporte_rien() {
        // Attendu -100, etalonnage -20, donc -120 planchés a -100.
        let r = calculer(&joueur(4, 50), &duel(true, 4, 0));
        assert_eq!(r.delta, 0);
        assert_eq!(r.rang_apres, r.rang_avant);
        // Et l'interface a de quoi l'expliquer plutot que d'afficher
        // un zero nu.
        assert!(r.effets.iter().any(|e| e.regle == Regle::Attendu));
    }

    #[test]
    fn une_defaite_tres_protegee_ne_rapporte_jamais_de_points() {
        // Consolation -100, etalonnage -20 : le plancher a -100 % evite
        // qu'une perte devienne un gain.
        let r = calculer(&joueur(0, 50), &duel(false, 0, 4));
        assert_eq!(r.delta, 0);
        assert_eq!(r.rang_apres, r.rang_avant);
    }

    #[test]
    fn le_plafond_ne_mord_pas_sur_le_jeu_ordinaire() {
        // Le meilleur match possible reste tres en dessous du plafond,
        // ce qui est le signe qu'il est bien place.
        let mut meilleur = 0;
        for consecutifs in 1..=8 {
            for mien in 0..=4u8 {
                for adverse in 0..=4u8 {
                    let mut j = joueur(mien.max(1), 50);
                    j.consecutifs = consecutifs;
                    let r = calculer(&j, &duel(true, mien, adverse));
                    meilleur = meilleur.max(r.delta);
                }
            }
        }
        assert!(
            meilleur < PLAFOND_PAR_MATCH,
            "le meilleur match rend {meilleur}, le plafond est {PLAFOND_PAR_MATCH}"
        );
        assert!(meilleur >= 30, "le meilleur match ne rend que {meilleur}");
    }

    // -----------------------------------------------------------------
    //  Les deux protections
    // -----------------------------------------------------------------

    #[test]
    fn le_debutant_ne_perd_rien() {
        let mut j = joueur(0, 30);
        j.deja_monte = false;
        let r = calculer(&j, &duel(false, 0, 0));
        assert_eq!(r.delta, 0);
        assert_eq!(r.protection, Some(Protection::Debutant));
        // Mais il gagne normalement.
        assert_eq!(calculer(&j, &duel(true, 0, 0)).delta, GAIN_BASE);
    }

    #[test]
    fn un_bronze_retrograde_perd_bel_et_bien() {
        // « Un joueur Silver retrograde en Bronze pourra perdre des points. »
        let mut j = joueur(0, 30);
        j.deja_monte = true;
        let r = calculer(&j, &duel(false, 0, 0));
        assert_eq!(r.delta, -PERTE_BASE);
        assert_eq!(r.protection, None);
    }

    #[test]
    fn la_protection_de_retrogradation_laisse_a_zero_puis_cede() {
        // L'exemple de Flow : quinze points, une defaite a trente.
        let j = Joueur {
            rang: rang(2, 15),
            consecutifs: 1,
            deja_monte: true,
            vient_de_monter: false,
        };
        let r = calculer(&j, &duel(false, 2, 2));
        assert_eq!(r.rang_apres, rang(2, 0));
        assert_eq!(r.protection, Some(Protection::Retrogradation));
        assert!(!r.retrogradation);
        // Le delta rendu est la perte reelle, pas celle qu'on visait.
        assert_eq!(r.delta, -15);

        // « il ne sera retrograde uniquement s'il perd encore ».
        let encore = Joueur {
            rang: rang(2, 0),
            ..j
        };
        let r2 = calculer(&encore, &duel(false, 2, 2));
        assert!(r2.retrogradation);
        assert_eq!(
            r2.rang_apres,
            rang(1, POINTS_PAR_PALIER - PERTE_BASE as u32)
        );
        assert_eq!(r2.protection, None);
    }

    #[test]
    fn on_ne_descend_jamais_sous_bronze_zero() {
        let j = Joueur {
            rang: rang(0, 0),
            consecutifs: 1,
            deja_monte: true,
            vient_de_monter: false,
        };
        let r = calculer(&j, &duel(false, 0, 0));
        assert_eq!(r.rang_apres, rang(0, 0));
        assert_eq!(r.delta, 0);
    }

    #[test]
    fn la_montee_de_palier_reporte_le_surplus() {
        let j = joueur(1, 90);
        let r = calculer(&j, &duel(true, 1, 1));
        assert!(r.promotion);
        // 90 + 22 = 112, donc Or a 12.
        assert_eq!(r.rang_apres, rang(2, 12));
    }

    #[test]
    fn on_ne_monte_pas_diamant_aux_points() {
        // Diamant est le top dix, donc une place et non un total. Le hub
        // l'attribue, ce module s'arrete a Platine.
        let j = joueur(3, 95);
        let r = calculer(&j, &duel(true, 3, 3));
        assert_eq!(r.rang_apres.palier, Palier::Platine);
        assert_eq!(r.rang_apres.points, POINTS_PAR_PALIER - 1);
        assert!(!r.promotion);
    }

    #[test]
    fn un_diamant_garde_son_palier_en_gagnant() {
        // Il ne doit surtout pas redescendre Platine parce qu'il a gagne.
        let j = joueur(4, 50);
        let r = calculer(&j, &duel(true, 4, 4));
        assert_eq!(r.rang_apres.palier, Palier::Diamant);
        assert!(!r.retrogradation);
    }

    #[test]
    fn un_diamant_continue_de_monter_dans_son_palier() {
        // Ses points departagent les dix premiers. Les bloquer figerait
        // le haut du classement, ce qui serait le pire endroit ou le
        // figer.
        let j = joueur(4, 50);
        let r = calculer(&j, &duel(true, 4, 4));
        assert_eq!(r.delta, GAIN_BASE);
        assert_eq!(r.rang_apres, rang(4, 50 + GAIN_BASE as u32));
        // Et il s'arrete quand meme au sommet de son palier.
        let haut = joueur(4, 95);
        assert_eq!(
            calculer(&haut, &duel(true, 4, 4)).rang_apres,
            rang(4, POINTS_PAR_PALIER - 1)
        );
    }

    // -----------------------------------------------------------------
    //  Le palier moyen d'une equipe
    // -----------------------------------------------------------------

    #[test]
    fn le_palier_moyen_se_calcule_sur_le_score_continu() {
        // Trois Bronze presque Argent valent Argent en moyenne, la ou une
        // moyenne d'indices aurait dit Bronze.
        assert_eq!(palier_moyen(&[rang(0, 90), rang(0, 90), rang(0, 90)]), 1);
        assert_eq!(palier_moyen(&[rang(0, 5), rang(0, 5), rang(0, 5)]), 0);
        // Un melange se range la ou il doit.
        assert_eq!(palier_moyen(&[rang(0, 0), rang(2, 0), rang(4, 0)]), 2);
        assert_eq!(palier_moyen(&[]), 0);
    }

    // -----------------------------------------------------------------
    //  L'equilibrage
    // -----------------------------------------------------------------

    #[test]
    fn l_equilibrage_trouve_la_meilleure_repartition() {
        // Six joueurs, deux de chaque niveau : la seule repartition
        // parfaite met un de chaque de chaque cote.
        let joueurs = [
            rang(0, 0),
            rang(0, 0),
            rang(2, 0),
            rang(2, 0),
            rang(4, 0),
            rang(4, 0),
        ];
        let e = equilibrer(&joueurs).unwrap();
        assert_eq!(e.ecart, 0);
        assert_eq!(e.equipe_a.len(), 3);
        assert_eq!(e.equipe_b.len(), 3);
        let somme = |c: &Vec<usize>| -> u32 { c.iter().map(|&i| joueurs[i].score()).sum() };
        assert_eq!(somme(&e.equipe_a), somme(&e.equipe_b));
    }

    #[test]
    fn l_equilibrage_fait_au_mieux_quand_la_perfection_est_impossible() {
        // Cinq Bronze et un Diamant : l'ecart minimal est celui du
        // Diamant moins les deux Bronze qui l'accompagnent.
        let joueurs = [
            rang(0, 0),
            rang(0, 10),
            rang(0, 20),
            rang(0, 30),
            rang(0, 40),
            rang(4, 0),
        ];
        let e = equilibrer(&joueurs).unwrap();
        // On verifie qu'aucune autre repartition ne fait mieux.
        let scores: Vec<i64> = joueurs.iter().map(|r| r.score() as i64).collect();
        let total: i64 = scores.iter().sum();
        let mut minimum = i64::MAX;
        for masque in 0u32..64 {
            if masque.count_ones() != 3 {
                continue;
            }
            let somme: i64 = (0..6)
                .filter(|i| masque >> i & 1 == 1)
                .map(|i| scores[i])
                .sum();
            minimum = minimum.min((somme * 2 - total).abs());
        }
        assert_eq!(e.ecart as i64, minimum);
    }

    #[test]
    fn l_equilibrage_est_deterministe() {
        let joueurs = [
            rang(1, 20),
            rang(3, 40),
            rang(0, 60),
            rang(2, 80),
            rang(4, 10),
            rang(1, 90),
        ];
        let premier = equilibrer(&joueurs).unwrap();
        for _ in 0..5 {
            assert_eq!(equilibrer(&joueurs).unwrap(), premier);
        }
    }

    #[test]
    fn l_equilibrage_refuse_ce_qu_il_ne_peut_pas_partager() {
        assert!(equilibrer(&[]).is_none());
        assert!(equilibrer(&[rang(0, 0)]).is_none());
        assert!(equilibrer(&[rang(0, 0), rang(0, 0), rang(0, 0)]).is_none());
        let trop: Vec<Rang> = (0..JOUEURS_MAX_EQUILIBRAGE + 2)
            .map(|_| rang(0, 0))
            .collect();
        assert!(equilibrer(&trop).is_none());
    }

    #[test]
    fn l_equilibrage_rend_deux_equipes_completes_et_sans_doublon() {
        let joueurs: Vec<Rang> = (0..8).map(|i| rang((i % 5) as u8, i * 7)).collect();
        let e = equilibrer(&joueurs).unwrap();
        let mut tous: Vec<usize> = e
            .equipe_a
            .iter()
            .chain(e.equipe_b.iter())
            .copied()
            .collect();
        tous.sort_unstable();
        assert_eq!(tous, (0..8).collect::<Vec<_>>());
        assert_eq!(e.equipe_a.len(), e.equipe_b.len());
    }
}
