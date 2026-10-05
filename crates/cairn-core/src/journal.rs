//! Le journal, cote lecture.
//!
//! **Ce module n'ouvre aucun fichier.** `cairn-app` rend le texte, ce
//! module le decoupe, le classe, l'anonymise, le compte et le filtre.
//! Tout est pur, donc tout se teste sans fenetre et sans disque, et
//! l'anonymisation en particulier merite d'etre testee : c'est elle qui
//! decide de ce qui sort de la machine de quelqu'un.
//!
//! **Le format d'ecriture a change au lot 23.** Une ligne s'ecrit
//! desormais `horodatage | NIVEAU | message`, la ou elle s'ecrivait
//! `horodatage | message`. Ce module lit les deux, et il le doit : le
//! fichier qui est deja sur la machine de Flow et sur celle de son
//! testeur est ecrit dans l'ancien format, et un lecteur qui le
//! refuserait serait inutile le jour ou il servirait.

use serde::{Deserialize, Serialize};

/// Ce qu'une ligne annonce.
///
/// Trois niveaux et pas cinq. Un journal de launcher n'a pas de `trace`
/// ni de `debug` a montrer : ce qui est ecrit ici est ecrit pour etre lu
/// par quelqu'un qui cherche pourquoi son jeu ne s'est pas lance, pas
/// par un profileur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Niveau {
    /// Le cours normal des choses.
    Info,
    /// Quelque chose n'a pas marche mais Cairn a continue autrement.
    Alerte,
    /// Une action demandee n'a pas eu lieu.
    Erreur,
}

impl Niveau {
    /// L'etiquette ecrite dans le fichier. En majuscules, pour qu'une
    /// ligne reste lisible a l'oeil nu dans un editeur de texte.
    pub fn etiquette(self) -> &'static str {
        match self {
            Niveau::Info => "INFO",
            Niveau::Alerte => "ALERTE",
            Niveau::Erreur => "ERREUR",
        }
    }

    /// La cle que l'interface manipule.
    pub fn cle(self) -> &'static str {
        match self {
            Niveau::Info => "info",
            Niveau::Alerte => "alerte",
            Niveau::Erreur => "erreur",
        }
    }

    /// Reconnait une etiquette, quelle que soit sa casse.
    pub fn depuis_etiquette(valeur: &str) -> Option<Niveau> {
        let v = valeur.trim();
        if v.eq_ignore_ascii_case("INFO") {
            Some(Niveau::Info)
        } else if v.eq_ignore_ascii_case("ALERTE") {
            Some(Niveau::Alerte)
        } else if v.eq_ignore_ascii_case("ERREUR") {
            Some(Niveau::Erreur)
        } else {
            None
        }
    }

    /// Reconnait une cle venue de l'interface.
    pub fn depuis_cle(valeur: &str) -> Option<Niveau> {
        match valeur {
            "info" => Some(Niveau::Info),
            "alerte" => Some(Niveau::Alerte),
            "erreur" => Some(Niveau::Erreur),
            _ => None,
        }
    }

    /// Les trois, dans l'ordre ou l'interface les propose.
    pub fn tous() -> [Niveau; 3] {
        [Niveau::Info, Niveau::Alerte, Niveau::Erreur]
    }
}

/// Une ligne du journal, telle qu'elle sera affichee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ligne {
    /// Son rang dans le fichier, a partir de 1.
    ///
    /// Il sert de cle stable a l'affichage et de repere quand quelqu'un
    /// decrit ce qu'il voit. Il ne se recalcule pas apres un filtre :
    /// « la ligne 214 » doit designer la meme ligne dans les deux vues.
    pub numero: usize,
    /// Tel qu'il est ecrit dans le fichier. Vide sur une ligne orpheline.
    pub horodatage: String,
    pub niveau: Niveau,
    pub message: String,
}

/// Combien de lignes de chaque niveau, avant filtrage.
///
/// Comptees sur le fichier entier et pas sur ce qui est affiche : le
/// compte sert a dire « il y a trois erreurs la-dedans », ce qui n'aurait
/// aucun sens s'il descendait a zero des qu'on filtre sur Info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Comptes {
    pub info: usize,
    pub alerte: usize,
    pub erreur: usize,
}

impl Comptes {
    pub fn total(self) -> usize {
        self.info + self.alerte + self.erreur
    }
}

/// Ce que l'interface recoit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Vue {
    /// Les lignes retenues par le filtre.
    pub lignes: Vec<Ligne>,
    /// Les comptes du fichier entier.
    pub comptes: Comptes,
    /// Vrai quand les chemins ont ete masques.
    pub anonyme: bool,
    /// Combien de lignes retenues par le filtre ont ete ecartees par la
    /// limite, donc combien manquent **avant** la premiere affichee.
    pub tronquees: usize,
}

/// Le texte qui remplace un nom d'utilisateur dans un chemin.
pub const MASQUE_UTILISATEUR: &str = "<utilisateur>";
/// Le texte qui remplace une adresse.
pub const MASQUE_ADRESSE: &str = "<adresse>";
/// Le texte qui remplace une adresse de courriel.
pub const MASQUE_COURRIEL: &str = "<courriel>";

/// Les dossiers apres lesquels vient un nom de personne.
///
/// En minuscules, la comparaison se fait sans tenir compte de la casse.
const MARQUEURS: [&str; 4] = [
    "\\users\\",
    "/users/",
    "/home/",
    "\\documents and settings\\",
];

// ---------------------------------------------------------------- analyse

/// Decoupe le texte du journal en lignes classees.
///
/// **Une ligne qui ne commence pas par un horodatage est rattachee a la
/// precedente.** C'est le cas des traces d'erreur sur plusieurs lignes :
/// les afficher comme des entrees separees, sans heure et sans niveau,
/// les rendrait illisibles et fausserait les comptes.
pub fn analyser(texte: &str) -> Vec<Ligne> {
    let mut lignes: Vec<Ligne> = Vec::new();
    for brute in texte.lines() {
        let brute = brute.trim_end_matches('\r');
        if brute.is_empty() {
            continue;
        }
        match decouper(brute) {
            Some((horodatage, niveau, message)) => lignes.push(Ligne {
                numero: lignes.len() + 1,
                horodatage,
                niveau,
                message,
            }),
            None => match lignes.last_mut() {
                Some(precedente) => {
                    precedente.message.push('\n');
                    precedente.message.push_str(brute);
                }
                // Un fichier qui commence au milieu d'une entree, ce qui
                // arrive apres une rotation. On la garde plutot que de la
                // perdre, sans heure et au niveau le plus neutre.
                None => lignes.push(Ligne {
                    numero: 1,
                    horodatage: String::new(),
                    niveau: Niveau::Info,
                    message: brute.to_string(),
                }),
            },
        }
    }
    lignes
}

/// Rend `None` quand la ligne n'est pas une entree, donc quand elle ne
/// commence pas par un horodatage suivi du separateur.
fn decouper(brute: &str) -> Option<(String, Niveau, String)> {
    let (debut, reste) = brute.split_once(" | ")?;
    if !ressemble_a_un_horodatage(debut) {
        return None;
    }
    // Nouveau format : l'etiquette de niveau vient juste apres.
    if let Some((etiquette, message)) = reste.split_once(" | ") {
        if let Some(niveau) = Niveau::depuis_etiquette(etiquette) {
            return Some((debut.to_string(), niveau, message.to_string()));
        }
    }
    // Ancien format. Le seul indice est le mot ERREUR que les messages
    // d'alors portaient a la main, quand ils le portaient. Tout le reste
    // passe en Info, et c'est assume : deviner un niveau a partir de
    // tournures comme « n'a pas pu » reviendrait a inventer une
    // information que le fichier ne contient pas.
    let niveau = if reste.trim_start().starts_with("ERREUR") {
        Niveau::Erreur
    } else {
        Niveau::Info
    };
    Some((debut.to_string(), niveau, reste.to_string()))
}

/// `2026-09-30 12:34:56.789`, avec ou sans les millisecondes.
fn ressemble_a_un_horodatage(valeur: &str) -> bool {
    let o = valeur.as_bytes();
    if o.len() < 19 {
        return false;
    }
    let chiffre = |i: usize| o[i].is_ascii_digit();
    (0..4).all(chiffre)
        && o[4] == b'-'
        && chiffre(5)
        && chiffre(6)
        && o[7] == b'-'
        && chiffre(8)
        && chiffre(9)
        && o[10] == b' '
        && chiffre(11)
        && chiffre(12)
        && o[13] == b':'
        && chiffre(14)
        && chiffre(15)
        && o[16] == b':'
        && chiffre(17)
        && chiffre(18)
}

// --------------------------------------------------------- anonymisation

/// Efface d'un texte ce qui designe une personne ou une machine.
///
/// **Trois choses, et trois seulement** : le nom d'utilisateur dans un
/// chemin, les adresses IP, les adresses de courriel. Le reste est
/// conserve, et c'est voulu : un rapport de bogue dont on a efface le nom
/// du serveur et le chemin du jeu ne sert plus a rien.
///
/// Sur le poste de Flow, le dossier utilisateur porte a la fois son
/// pseudo et le nom de sa machine, `Xirdrak.DESKTOP-3OOD2ME`. Masquer le
/// segment du chemin les efface donc tous les deux d'un coup, sans avoir
/// a connaitre ni l'un ni l'autre.
pub fn anonymiser(texte: &str) -> String {
    masquer_courriels(&masquer_adresses(&masquer_utilisateurs(texte)))
}

/// `C:\Users\Machin\...` devient `C:\Users\<utilisateur>\...`.
///
/// Le segment s'arrete au prochain separateur, ou a la fin de la ligne
/// s'il n'y en a pas. **Cette seconde branche masque parfois un peu trop**
/// quand une phrase suit un chemin sans separateur final. C'est le bon
/// sens de l'erreur : trop masquer gene la lecture, pas assez publie le
/// nom de quelqu'un.
fn masquer_utilisateurs(texte: &str) -> String {
    let o = texte.as_bytes();
    let mut sortie = String::with_capacity(texte.len());
    let mut copie = 0usize;
    let mut i = 0usize;
    while i < o.len() {
        let Some(taille) = marqueur_a(o, i) else {
            i += 1;
            continue;
        };
        let apres = i + taille;
        sortie.push_str(&texte[copie..apres]);
        let mut j = apres;
        // Le segment ne franchit jamais une fin de ligne : le journal est
        // analyse d'un bloc, et un chemin ne se poursuit pas sur la ligne
        // suivante.
        while j < o.len() && o[j] != b'\\' && o[j] != b'/' && o[j] != b'\n' && o[j] != b'\r' {
            j += 1;
        }
        if j > apres {
            sortie.push_str(MASQUE_UTILISATEUR);
        }
        copie = j;
        i = j;
    }
    sortie.push_str(&texte[copie..]);
    sortie
}

/// La taille du marqueur qui commence a `i`, sans tenir compte de la casse.
fn marqueur_a(o: &[u8], i: usize) -> Option<usize> {
    for marqueur in MARQUEURS {
        let m = marqueur.as_bytes();
        if i + m.len() <= o.len() && o[i..i + m.len()].eq_ignore_ascii_case(m) {
            return Some(m.len());
        }
    }
    None
}

/// Masque les adresses IPv4, avec leur port s'il y en a un.
///
/// Quatre groupes de un a trois chiffres, chacun au plus 255. La
/// contrainte des quatre groupes est ce qui empeche d'avaler un numero de
/// version : `Cairn 0.1.0 demarre` n'en a que trois, et cette ligne est
/// la premiere du fichier.
fn masquer_adresses(texte: &str) -> String {
    let o = texte.as_bytes();
    let mut sortie = String::with_capacity(texte.len());
    let mut copie = 0usize;
    let mut i = 0usize;
    while i < o.len() {
        // Ne demarrer qu'au debut d'un nombre, sinon `10.0.0.1` serait
        // aussi essaye depuis son `0`.
        if !o[i].is_ascii_digit() || (i > 0 && (o[i - 1].is_ascii_digit() || o[i - 1] == b'.')) {
            i += 1;
            continue;
        }
        let Some(mut fin) = adresse_a(o, i) else {
            i += 1;
            continue;
        };
        sortie.push_str(&texte[copie..i]);
        sortie.push_str(MASQUE_ADRESSE);
        // Le port fait partie de l'adresse : le garder livrerait la
        // moitie de ce qu'on vient de masquer.
        if fin < o.len() && o[fin] == b':' {
            let mut k = fin + 1;
            while k < o.len() && o[k].is_ascii_digit() {
                k += 1;
            }
            if k > fin + 1 {
                fin = k;
            }
        }
        copie = fin;
        i = fin;
    }
    sortie.push_str(&texte[copie..]);
    sortie
}

/// La fin de l'adresse qui commence a `i`, ou `None`.
fn adresse_a(o: &[u8], i: usize) -> Option<usize> {
    let mut position = i;
    for groupe in 0..4 {
        if groupe > 0 {
            if position >= o.len() || o[position] != b'.' {
                return None;
            }
            position += 1;
        }
        let debut = position;
        let mut valeur: u32 = 0;
        while position < o.len() && o[position].is_ascii_digit() && position - debut < 3 {
            valeur = valeur * 10 + u32::from(o[position] - b'0');
            position += 1;
        }
        if position == debut || valeur > 255 {
            return None;
        }
    }
    // Un cinquieme groupe veut dire qu'on est dans autre chose.
    if position < o.len() && (o[position] == b'.' || o[position].is_ascii_digit()) {
        return None;
    }
    Some(position)
}

/// Masque les adresses de courriel.
///
/// **Le point dans le domaine est obligatoire**, et ce n'est pas un
/// detail : un identifiant de title pack s'ecrit `obstacle@smokegun`,
/// avec une arobase et sans point. Sans cette condition, le lecteur
/// effacerait de ses propres lignes l'information qu'elles servent a
/// porter.
fn masquer_courriels(texte: &str) -> String {
    let o = texte.as_bytes();
    let mut sortie = String::with_capacity(texte.len());
    let mut copie = 0usize;
    let mut i = 0usize;
    while i < o.len() {
        if o[i] != b'@' {
            i += 1;
            continue;
        }
        let debut = debut_de_courriel(o, i);
        let fin = fin_de_courriel(o, i);
        if debut == i || fin == i + 1 || !domaine_plausible(&o[i + 1..fin]) {
            i += 1;
            continue;
        }
        sortie.push_str(&texte[copie..debut]);
        sortie.push_str(MASQUE_COURRIEL);
        copie = fin;
        i = fin;
    }
    sortie.push_str(&texte[copie..]);
    sortie
}

fn caractere_de_courriel(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-' | b'+')
}

fn debut_de_courriel(o: &[u8], arobase: usize) -> usize {
    let mut debut = arobase;
    while debut > 0 && caractere_de_courriel(o[debut - 1]) {
        debut -= 1;
    }
    debut
}

fn fin_de_courriel(o: &[u8], arobase: usize) -> usize {
    let mut fin = arobase + 1;
    while fin < o.len() && caractere_de_courriel(o[fin]) {
        fin += 1;
    }
    // Un point final appartient a la phrase, pas au domaine.
    while fin > arobase + 1 && o[fin - 1] == b'.' {
        fin -= 1;
    }
    fin
}

fn domaine_plausible(domaine: &[u8]) -> bool {
    let Some(point) = domaine.iter().rposition(|c| *c == b'.') else {
        return false;
    };
    let extension = &domaine[point + 1..];
    extension.len() >= 2 && extension.iter().all(|c| c.is_ascii_alphabetic())
}

// ------------------------------------------------------- comptes, filtre

/// Compte les lignes de chaque niveau.
pub fn compter(lignes: &[Ligne]) -> Comptes {
    let mut comptes = Comptes::default();
    for ligne in lignes {
        match ligne.niveau {
            Niveau::Info => comptes.info += 1,
            Niveau::Alerte => comptes.alerte += 1,
            Niveau::Erreur => comptes.erreur += 1,
        }
    }
    comptes
}

/// Garde les lignes du bon niveau qui repondent a la recherche.
///
/// La recherche est celle de la liste des serveurs, `recherche::correspond` :
/// sans accents, sans casse, et chaque mot doit se trouver quelque part.
/// Deux barres de recherche qui ne se comportent pas pareil dans la meme
/// application, c'est une de trop a apprendre.
///
/// **Une liste de niveaux vide ne filtre rien.** Un filtre ou l'on a tout
/// decoche doit montrer une liste vide et pas la liste entiere : c'est ce
/// que l'utilisateur a demande, et une liste qui se remplit quand on
/// decoche la derniere case est incomprehensible.
pub fn filtrer(lignes: Vec<Ligne>, niveaux: &[Niveau], recherche: &str) -> Vec<Ligne> {
    lignes
        .into_iter()
        .filter(|ligne| {
            niveaux.contains(&ligne.niveau)
                && crate::recherche::correspond(&[&ligne.message, &ligne.horodatage], recherche)
        })
        .collect()
}

/// Le chemin complet : analyse, anonymisation, comptes, filtre.
///
/// L'anonymisation vient **avant** les comptes et la recherche, pour que
/// ce qu'on cherche soit ce qu'on voit. Chercher son propre nom
/// d'utilisateur dans un journal anonymise ne doit rien donner, sinon le
/// masque n'en est pas un.
pub fn preparer(
    texte: &str,
    niveaux: &[Niveau],
    recherche: &str,
    anonyme: bool,
    limite: usize,
) -> Vue {
    let texte = if anonyme {
        anonymiser(texte)
    } else {
        texte.to_string()
    };
    let lignes = analyser(&texte);
    let comptes = compter(&lignes);
    let mut gardees = filtrer(lignes, niveaux, recherche);
    // La limite garde la FIN et pas le debut. Un journal se lit par ce
    // qui vient de se passer : tronquer par la fin donnerait les mille
    // premieres lignes d'une session vieille de trois semaines, et
    // l'erreur d'il y a deux minutes ne serait nulle part.
    let tronquees = if limite > 0 && gardees.len() > limite {
        let trop = gardees.len() - limite;
        gardees.drain(..trop);
        trop
    } else {
        0
    };
    Vue {
        lignes: gardees,
        comptes,
        anonyme,
        tronquees,
    }
}

/// Met en forme un rapport a envoyer.
///
/// **Toujours anonymise, quel que soit l'affichage.** C'est le seul
/// endroit du module ou l'on ne suit pas le choix de l'utilisateur :
/// regarder ses propres chemins sur sa propre machine le regarde, les
/// publier dans un salon de discussion n'est plus la meme chose, et
/// personne ne relit mille lignes avant de les coller.
///
/// L'entete est fabriquee par `cairn-app`, qui seul connait la version,
/// le systeme et le numero de contrat.
pub fn exporter(entete: &[(&str, String)], texte: &str) -> String {
    let mut sortie = String::new();
    sortie.push_str("Rapport Cairn\n");
    for (cle, valeur) in entete {
        sortie.push_str(cle);
        sortie.push_str(" : ");
        sortie.push_str(valeur);
        sortie.push('\n');
    }
    sortie.push_str(
        "\nLes chemins, les adresses et les courriels ont ete remplaces\n\
         automatiquement. Relis quand meme avant d'envoyer.\n\n",
    );
    sortie.push_str(&anonymiser(texte));
    if !sortie.ends_with('\n') {
        sortie.push('\n');
    }
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------ analyse

    #[test]
    fn le_nouveau_format_donne_les_trois_morceaux() {
        let lignes = analyser("2026-09-30 12:34:56.789 | ERREUR | ManiaPlanet.exe introuvable.");
        assert_eq!(lignes.len(), 1);
        assert_eq!(lignes[0].horodatage, "2026-09-30 12:34:56.789");
        assert_eq!(lignes[0].niveau, Niveau::Erreur);
        assert_eq!(lignes[0].message, "ManiaPlanet.exe introuvable.");
        assert_eq!(lignes[0].numero, 1);
    }

    #[test]
    fn l_ancien_format_reste_lisible() {
        // Le fichier deja present chez Flow et chez son testeur.
        let lignes = analyser(
            "2026-09-28 20:01:02.003 | Cairn 0.1.0 demarre sur windows / x86_64.\n\
             2026-09-28 20:01:02.100 | ERREUR : dossier de reglages (acces refuse).",
        );
        assert_eq!(lignes.len(), 2);
        assert_eq!(lignes[0].niveau, Niveau::Info);
        assert_eq!(
            lignes[0].message,
            "Cairn 0.1.0 demarre sur windows / x86_64."
        );
        assert_eq!(lignes[1].niveau, Niveau::Erreur);
    }

    #[test]
    fn une_ligne_sans_horodatage_rejoint_la_precedente() {
        let lignes = analyser(
            "2026-09-30 12:00:00.000 | ERREUR | Le lancement a echoue :\n\
             au fichier jeu.rs, ligne 42\n\
             2026-09-30 12:00:01.000 | INFO | Fermeture demandee.",
        );
        assert_eq!(lignes.len(), 2);
        assert!(lignes[0].message.contains("ligne 42"));
        assert_eq!(lignes[1].numero, 2);
    }

    #[test]
    fn un_fichier_qui_commence_au_milieu_ne_perd_pas_sa_premiere_ligne() {
        let lignes = analyser("au fichier jeu.rs, ligne 42\n2026-09-30 12:00:01.000 | INFO | Ok.");
        assert_eq!(lignes.len(), 2);
        assert_eq!(lignes[0].horodatage, "");
        assert_eq!(lignes[0].message, "au fichier jeu.rs, ligne 42");
    }

    #[test]
    fn un_message_qui_contient_le_separateur_n_est_pas_coupe() {
        let lignes = analyser("2026-09-30 12:00:00.000 | INFO | a | b | c");
        assert_eq!(lignes[0].message, "a | b | c");
    }

    #[test]
    fn une_etiquette_inconnue_ne_mange_pas_le_message() {
        // Sinon un message qui commence par un mot suivi de « | » verrait
        // ce mot disparaitre de l'affichage.
        let lignes = analyser("2026-09-30 12:00:00.000 | Jeu trouve | version 3");
        assert_eq!(lignes[0].niveau, Niveau::Info);
        assert_eq!(lignes[0].message, "Jeu trouve | version 3");
    }

    #[test]
    fn les_numeros_suivent_le_fichier_et_pas_le_filtre() {
        let lignes = analyser(
            "2026-09-30 12:00:00.000 | INFO | un\n\
             2026-09-30 12:00:01.000 | ERREUR | deux\n\
             2026-09-30 12:00:02.000 | INFO | trois",
        );
        let gardees = filtrer(lignes, &[Niveau::Erreur], "");
        assert_eq!(gardees.len(), 1);
        assert_eq!(gardees[0].numero, 2);
    }

    // ----------------------------------------------------- anonymisation

    #[test]
    fn le_dossier_utilisateur_de_windows_est_masque() {
        let avant = "Jeu trouve : C:\\Users\\Xirdrak.DESKTOP-3OOD2ME\\Documents\\ManiaPlanet";
        let apres = anonymiser(avant);
        assert!(!apres.contains("Xirdrak"));
        assert!(!apres.contains("DESKTOP-3OOD2ME"));
        assert!(apres.contains("C:\\Users\\<utilisateur>\\Documents\\ManiaPlanet"));
    }

    #[test]
    fn la_casse_du_dossier_ne_change_rien() {
        for variante in [
            "C:\\USERS\\Bob\\x",
            "c:\\users\\Bob\\x",
            "C:\\UsErS\\Bob\\x",
        ] {
            assert!(!anonymiser(variante).contains("Bob"), "{variante}");
        }
    }

    #[test]
    fn un_nom_avec_une_espace_est_masque_en_entier() {
        let apres = anonymiser("dans C:\\Users\\Jean Dupont\\Profiles ok");
        assert!(!apres.contains("Dupont"));
        assert!(apres.contains("\\Profiles ok"));
    }

    #[test]
    fn un_chemin_en_fin_de_ligne_est_masque() {
        let apres = anonymiser("dossier : C:\\Users\\Bob");
        assert_eq!(apres, "dossier : C:\\Users\\<utilisateur>");
    }

    #[test]
    fn le_masque_ne_franchit_pas_la_fin_de_ligne() {
        let apres = anonymiser("C:\\Users\\Bob\n2026-09-30 12:00:00.000 | INFO | suite");
        assert!(apres.contains("<utilisateur>\n2026-09-30"));
    }

    #[test]
    fn les_chemins_systeme_sont_conserves() {
        // Ils ne designent personne, et les effacer rendrait le rapport
        // inutilisable.
        let avant = "ERREUR : C:\\Program Files\\ManiaPlanet\\ManiaPlanet.exe n'existe pas.";
        assert_eq!(anonymiser(avant), avant);
    }

    #[test]
    fn les_dossiers_des_autres_systemes_sont_couverts() {
        assert!(!anonymiser("/home/flow/jeux").contains("flow"));
        assert!(!anonymiser("/Users/flow/jeux").contains("flow"));
        assert!(!anonymiser("C:\\Documents and Settings\\flow\\jeux").contains("flow"));
    }

    #[test]
    fn une_adresse_est_masquee_avec_son_port() {
        let apres = anonymiser("Connexion a 82.65.123.7:2350 refusee");
        assert_eq!(apres, "Connexion a <adresse> refusee");
    }

    #[test]
    fn un_numero_de_version_n_est_pas_une_adresse() {
        // La toute premiere ligne du journal porte la version. Si elle
        // etait masquee, le rapport perdrait ce qu'on lit en premier.
        let avant = "Cairn 0.1.0 demarre sur windows / x86_64.";
        assert_eq!(anonymiser(avant), avant);
    }

    #[test]
    fn un_groupe_trop_grand_n_est_pas_une_adresse() {
        let avant = "recu 300.1.2.3 octets";
        assert_eq!(anonymiser(avant), avant);
    }

    #[test]
    fn cinq_groupes_ne_sont_pas_une_adresse() {
        let avant = "sequence 1.2.3.4.5 vue";
        assert_eq!(anonymiser(avant), avant);
    }

    #[test]
    fn un_identifiant_de_title_pack_n_est_pas_un_courriel() {
        // Le controle qui compte : ces identifiants sont ecrits dans le
        // journal a chaque ouverture de titre, et ce sont eux qu'on lit
        // pour comprendre ce qui s'est passe.
        for titre in [
            "Ouverture du titre : obstacle@smokegun",
            "Ouverture du titre : SMStormElite@nadeolabs",
            "Ouverture du titre : GEs@guerro",
        ] {
            assert_eq!(anonymiser(titre), titre, "{titre}");
        }
    }

    #[test]
    fn un_courriel_est_masque() {
        let apres = anonymiser("contact : flow.marques@example.com, merci.");
        assert_eq!(apres, "contact : <courriel>, merci.");
    }

    #[test]
    fn l_anonymisation_ne_touche_pas_un_journal_ordinaire() {
        let avant = "2026-09-30 12:00:00.000 | INFO | 126 serveur(s) recu(s) en 2 page(s).";
        assert_eq!(anonymiser(avant), avant);
    }

    #[test]
    fn l_anonymisation_est_stable() {
        // La repasser ne doit rien changer : sinon un export repris deux
        // fois finirait par masquer ses propres masques.
        let une_fois = anonymiser("C:\\Users\\Bob\\x et 10.0.0.1:2350");
        assert_eq!(anonymiser(&une_fois), une_fois);
    }

    // -------------------------------------------------- comptes et filtre

    fn echantillon() -> &'static str {
        "2026-09-30 12:00:00.000 | INFO | Cairn demarre.\n\
         2026-09-30 12:00:01.000 | ALERTE | Reglages illisibles, valeurs par defaut.\n\
         2026-09-30 12:00:02.000 | ERREUR | ManiaPlanet.exe introuvable.\n\
         2026-09-30 12:00:03.000 | INFO | Fermeture demandee par l'interface."
    }

    #[test]
    fn les_comptes_portent_sur_le_fichier_entier() {
        let vue = preparer(echantillon(), &[Niveau::Erreur], "", true, 0);
        assert_eq!(vue.lignes.len(), 1);
        // Et pas 0, 0, 1 : le compte dit ce que contient le fichier.
        assert_eq!(vue.comptes.info, 2);
        assert_eq!(vue.comptes.alerte, 1);
        assert_eq!(vue.comptes.erreur, 1);
        assert_eq!(vue.comptes.total(), 4);
    }

    #[test]
    fn aucun_niveau_coche_ne_montre_rien() {
        let vue = preparer(echantillon(), &[], "", true, 0);
        assert!(vue.lignes.is_empty());
        assert_eq!(vue.comptes.total(), 4);
    }

    #[test]
    fn la_recherche_ignore_la_casse_et_les_accents() {
        let vue = preparer(echantillon(), &Niveau::tous(), "REGLAGES", true, 0);
        assert_eq!(vue.lignes.len(), 1);
        let vue = preparer(echantillon(), &Niveau::tous(), "réglages", true, 0);
        assert_eq!(vue.lignes.len(), 1);
    }

    #[test]
    fn la_recherche_porte_aussi_sur_l_heure() {
        let vue = preparer(echantillon(), &Niveau::tous(), "12:00:02", true, 0);
        assert_eq!(vue.lignes.len(), 1);
        assert_eq!(vue.lignes[0].niveau, Niveau::Erreur);
    }

    #[test]
    fn chercher_son_nom_dans_un_journal_anonymise_ne_donne_rien() {
        let texte = "2026-09-30 12:00:00.000 | INFO | Jeu trouve : C:\\Users\\Xirdrak\\jeux";
        let vue = preparer(texte, &Niveau::tous(), "Xirdrak", true, 0);
        assert!(vue.lignes.is_empty());
        let vue = preparer(texte, &Niveau::tous(), "Xirdrak", false, 0);
        assert_eq!(vue.lignes.len(), 1);
    }

    // ------------------------------------------------------------- export

    #[test]
    fn la_limite_garde_la_fin() {
        // Le controle qui compte : tronquer par le debut rendrait les
        // premieres lignes d'une vieille session et perdrait ce qui vient
        // d'arriver, ce qui est l'inverse de ce qu'on ouvre un journal
        // pour voir.
        let vue = preparer(echantillon(), &Niveau::tous(), "", true, 2);
        assert_eq!(vue.lignes.len(), 2);
        assert_eq!(vue.tronquees, 2);
        assert_eq!(vue.lignes[0].numero, 3);
        assert_eq!(vue.lignes[1].numero, 4);
        assert_eq!(vue.comptes.total(), 4);
    }

    #[test]
    fn une_limite_nulle_ne_tronque_rien() {
        let vue = preparer(echantillon(), &Niveau::tous(), "", true, 0);
        assert_eq!(vue.lignes.len(), 4);
        assert_eq!(vue.tronquees, 0);
    }

    #[test]
    fn une_limite_plus_grande_que_le_journal_ne_tronque_rien() {
        let vue = preparer(echantillon(), &Niveau::tous(), "", true, 500);
        assert_eq!(vue.lignes.len(), 4);
        assert_eq!(vue.tronquees, 0);
    }

    #[test]
    fn l_export_est_anonymise_meme_quand_l_affichage_ne_l_est_pas() {
        let texte = "2026-09-30 12:00:00.000 | INFO | C:\\Users\\Xirdrak\\jeux";
        let rapport = exporter(&[("Version", "0.1.0".to_string())], texte);
        assert!(!rapport.contains("Xirdrak"));
        assert!(rapport.contains("Version : 0.1.0"));
        assert!(rapport.ends_with('\n'));
    }

    #[test]
    fn les_etiquettes_font_l_aller_retour() {
        for niveau in Niveau::tous() {
            assert_eq!(Niveau::depuis_etiquette(niveau.etiquette()), Some(niveau));
            assert_eq!(Niveau::depuis_cle(niveau.cle()), Some(niveau));
        }
        assert_eq!(Niveau::depuis_cle("verbeux"), None);
    }
}
