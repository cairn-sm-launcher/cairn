//! Le fil d'actualites : son format, sa signature, et son markdown.
//!
//! **Tout est pur.** Ce module ne telecharge rien et n'ecrit rien :
//! `cairn-app` rend des octets, ce module dit s'il faut les croire, ce
//! qu'ils contiennent, et sous quelle forme l'interface doit les
//! dessiner. Un fil se teste donc entierement sans reseau.
//!
//! ## Pourquoi une dependance de plus dans cette caisse
//!
//! `cairn-core` n'avait que serde, et c'etait une fierte. `ed25519-dalek`
//! arrive ici et pas dans `cairn-app` pour la raison habituelle : la
//! verification d'une signature est exactement ce qui merite d'etre
//! teste, et ce qui merite un test vit ici. C'est du calcul pur, sans
//! entree-sortie et sans systeme, donc la regle de la caisse tient.
//!
//! ## Ce que la signature protege, et ce qu'elle ne protege pas
//!
//! Le HTTPS prouve que le fichier vient bien de l'hebergeur annonce. La
//! signature prouve qu'il vient de **l'auteur**, meme si le compte de
//! l'hebergement est pris. Les deux ne font pas double emploi.
//!
//! Elle ne protege pas contre le rejeu : quelqu'un qui controle
//! l'hebergement peut servir un ancien fil, correctement signe, pour
//! cacher une annonce recente. D'ou `plus_recent_que`, qui refuse un fil
//! anterieur a celui deja en cache.

use serde::{Deserialize, Serialize};

/// La version de format que cette version de Cairn sait lire.
///
/// Un fil qui annonce une version superieure n'est pas refuse : ses
/// articles sont lus avec ce qu'on comprend. Refuser serait prendre le
/// risque qu'une publication coupe les actualites de tous ceux qui n'ont
/// pas mis a jour, ce qui est l'inverse du but.
pub const VERSION_FIL: u32 = 1;

/// La taille d'une signature Ed25519, en octets.
const TAILLE_SIGNATURE: usize = 64;
/// La taille d'une cle publique Ed25519, en octets.
const TAILLE_CLE: usize = 32;

// ------------------------------------------------------------- le format

/// Le genre d'une actualite.
///
/// Ensemble ferme, avec un fourre-tout. **Une categorie inconnue ne fait
/// pas perdre l'article**, elle le range dans `Autre` : le jour ou une
/// nouvelle categorie est publiee, les Cairn plus anciens continuent de
/// montrer les articles au lieu de les laisser tomber. C'est la lecon du
/// numero de contrat, appliquee aux donnees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Categorie {
    /// Coupes, tournois, resultats.
    Tournoi,
    /// Mappacks, maps mises en avant.
    Maps,
    /// Modes de jeu, packs d'items, outils.
    Creation,
    /// Serveurs, mises a jour, sujets techniques.
    Technique,
    /// Entraide, questions, recherche de joueurs.
    Entraide,
    /// Tout le reste, et ce que Cairn ne connait pas encore.
    Autre,
}

impl Categorie {
    pub fn cle(self) -> &'static str {
        match self {
            Categorie::Tournoi => "tournoi",
            Categorie::Maps => "maps",
            Categorie::Creation => "creation",
            Categorie::Technique => "technique",
            Categorie::Entraide => "entraide",
            Categorie::Autre => "autre",
        }
    }

    /// Reconnait une cle. **Rend `Autre` et jamais `None`** : voir plus haut.
    pub fn depuis_cle(valeur: &str) -> Categorie {
        match valeur.trim().to_ascii_lowercase().as_str() {
            "tournoi" => Categorie::Tournoi,
            "maps" => Categorie::Maps,
            "creation" => Categorie::Creation,
            "technique" => Categorie::Technique,
            "entraide" => Categorie::Entraide,
            _ => Categorie::Autre,
        }
    }

    pub fn toutes() -> [Categorie; 6] {
        [
            Categorie::Tournoi,
            Categorie::Maps,
            Categorie::Creation,
            Categorie::Technique,
            Categorie::Entraide,
            Categorie::Autre,
        ]
    }
}

/// Un article, tel qu'il est publie.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Article {
    pub id: String,
    pub titre: String,
    /// Deserialisee en chaine puis rangee, pour que l'inconnu devienne
    /// `Autre` au lieu de faire echouer la lecture du fil entier.
    #[serde(default)]
    pub categorie: String,
    /// Secondes depuis 1970.
    pub date: i64,
    #[serde(default)]
    pub resume: String,
    #[serde(default)]
    pub corps: String,
    /// `fr`, `en`, ou vide quand l'article vaut dans les deux langues.
    #[serde(default)]
    pub langue: String,
    #[serde(default)]
    pub epingle: bool,
}

/// Le fil entier, tel qu'il est servi.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Fil {
    pub version: u32,
    /// Quand le fil a ete assemble. Sert au controle anti-rejeu.
    pub publie: i64,
    #[serde(default)]
    pub articles: Vec<Article>,
}

/// Un article pret a afficher : categorie rangee, corps decoupe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArticleRendu {
    pub id: String,
    pub titre: String,
    pub categorie: Categorie,
    pub date: i64,
    pub resume: String,
    pub langue: String,
    pub epingle: bool,
    pub corps: Vec<Bloc>,
}

/// Ce qui empeche le fil d'etre frais.
///
/// **Une cle et pas une phrase.** `cairn-core` et `cairn-app` sont ecrits
/// en francais, l'interface se lit dans deux langues, et une erreur rendue
/// toute faite depuis Rust arrive en francais sur un ecran anglais. Le
/// projet s'etait deja donne la regle pour la date du profil, « rendre une
/// date nue et pas une phrase, pour ne pas imposer sa langue a
/// l'interface » ; le chemin des erreurs l'avait oubliee. Le detail
/// technique, lui, va dans le journal, qui n'a qu'une langue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Souci {
    /// Le reseau n'a pas repondu, ou a repondu autre chose qu'un fil.
    Reseau,
    /// La signature ne correspond pas a la cle de Cairn.
    Signature,
    /// Le fil est arrive mais ne se lit pas.
    Illisible,
    /// Le fil recu est anterieur a celui deja garde.
    Rejeu,
    /// Le fil depasse la taille admise.
    TropGros,
    /// L'adresse du fil n'a pas encore ete renseignee.
    ///
    /// Ce n'est pas une panne, c'est une etape de mise en place qui
    /// reste a faire, et l'ecran doit le dire autrement.
    NonConfigure,
}

impl Souci {
    pub fn cle(self) -> &'static str {
        match self {
            Souci::Reseau => "reseau",
            Souci::Signature => "signature",
            Souci::Illisible => "illisible",
            Souci::Rejeu => "rejeu",
            Souci::TropGros => "tropgros",
            Souci::NonConfigure => "nonconfigure",
        }
    }
}

// ------------------------------------------------------------- signature

/// Verifie la signature d'un fil.
///
/// **Elle porte sur les octets tels qu'ils arrivent**, jamais sur un
/// JSON re-serialise. C'est le piege classique de la signature de JSON :
/// relire puis reecrire change l'ordre des cles et les espaces, donc la
/// signature ne colle plus, et on finit par « normaliser » le document,
/// ce qui ouvre autant de trous qu'il existe de facons de normaliser. Le
/// document signe est le fichier, point.
///
/// `signature` et `cle_publique` sont en hexadecimal, parce que c'est ce
/// qui se lit et se recopie sans ambiguite dans un fichier de
/// configuration ou un secret de publication.
pub fn signature_valable(octets: &[u8], signature: &str, cle_publique: &str) -> bool {
    let Some(sig) = depuis_hex::<TAILLE_SIGNATURE>(signature) else {
        return false;
    };
    let Some(cle) = depuis_hex::<TAILLE_CLE>(cle_publique) else {
        return false;
    };
    let Ok(verificateur) = ed25519_dalek::VerifyingKey::from_bytes(&cle) else {
        return false;
    };
    verificateur
        .verify_strict(octets, &ed25519_dalek::Signature::from_bytes(&sig))
        .is_ok()
}

/// Decode une chaine hexadecimale de taille exacte.
///
/// Ecrit a la main plutot que tire d'une caisse : vingt lignes contre une
/// dependance de plus, dans un module qui vient d'en ajouter une.
fn depuis_hex<const N: usize>(valeur: &str) -> Option<[u8; N]> {
    let valeur = valeur.trim();
    if valeur.len() != N * 2 {
        return None;
    }
    let mut sortie = [0u8; N];
    let octets = valeur.as_bytes();
    for (i, place) in sortie.iter_mut().enumerate() {
        let haut = chiffre_hex(octets[i * 2])?;
        let bas = chiffre_hex(octets[i * 2 + 1])?;
        *place = haut * 16 + bas;
    }
    Some(sortie)
}

fn chiffre_hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Vrai quand le fil recu est au moins aussi recent que celui en cache.
///
/// Protege du rejeu : une signature reste valable pour toujours, donc
/// quelqu'un qui controle l'hebergement pourrait reservir le fil de la
/// semaine derniere, parfaitement signe, pour faire disparaitre une
/// annonce. L'egalite est acceptee, sinon un fil republie a l'identique
/// serait refuse sans raison.
pub fn plus_recent_que(publie: i64, cache: Option<i64>) -> bool {
    match cache {
        None => true,
        Some(ancien) => publie >= ancien,
    }
}

// -------------------------------------------------------------- les liens

/// Vrai quand un lien peut etre ouvert dans le navigateur du systeme.
///
/// **Seulement `http` et `https`.** Le lien part dans un appel systeme
/// qui, sous Windows, sait aussi ouvrir un fichier, un dossier et un
/// programme : `file:///C:/Windows/...` ou un chemin UNC seraient donc
/// des liens cliquables vers la machine de celui qui lit. Un article est
/// ecrit par quelqu'un d'autre, c'est tout ce qu'il faut savoir pour
/// fermer cette porte au lieu de faire confiance.
///
/// Les caracteres de controle sont refuses en bloc : ils ne servent a
/// rien dans une adresse et sont la matiere premiere de la plupart des
/// contournements.
pub fn lien_autorise(lien: &str) -> bool {
    let lien = lien.trim();
    if lien.is_empty() || lien.len() > 2048 {
        return false;
    }
    if lien
        .chars()
        .any(|c| c.is_control() || c == '"' || c == '\'')
    {
        return false;
    }
    let minuscule = lien.to_ascii_lowercase();
    if !(minuscule.starts_with("http://") || minuscule.starts_with("https://")) {
        return false;
    }
    // Un hote doit exister apres le schema.
    let reste = &lien[minuscule.find("//").map(|i| i + 2).unwrap_or(lien.len())..];
    let hote = reste
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('.');
    // `@` dans l'hote sert a camoufler la vraie destination derriere un
    // faux identifiant, du genre `https://nadeo.com@mechant.fr`.
    !hote.is_empty() && !hote.contains('@') && hote.contains(|c: char| c.is_ascii_alphanumeric())
}

// ----------------------------------------------------------- le markdown

/// Un morceau de texte dans un paragraphe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "sorte", rename_all = "lowercase")]
pub enum Fragment {
    Texte {
        texte: String,
    },
    Gras {
        texte: String,
    },
    Italique {
        texte: String,
    },
    Code {
        texte: String,
    },
    Lien {
        texte: String,
        lien: String,
    },
    /// Une image. **Elle est rendue comme un lien et pas comme une
    /// image**, voir la note de `analyser`.
    Image {
        texte: String,
        lien: String,
    },
}

/// Un bloc d'article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "sorte", rename_all = "lowercase")]
pub enum Bloc {
    Titre {
        niveau: u8,
        texte: String,
    },
    Paragraphe {
        fragments: Vec<Fragment>,
    },
    Liste {
        ordonnee: bool,
        points: Vec<Vec<Fragment>>,
    },
    Citation {
        fragments: Vec<Fragment>,
    },
    Code {
        texte: String,
    },
    Separateur,
}

/// Decoupe un corps markdown en blocs.
///
/// **Rend une structure, jamais du HTML.** C'est la decision de securite
/// du lot : il n'y a nulle part dans Cairn une chaine HTML venue d'un
/// article, donc pas de `{@html}`, donc rien a assainir et rien a
/// oublier d'assainir. L'injection devient impossible par construction
/// plutot que par vigilance.
///
/// Le sous-ensemble couvert : titres de niveau 1 a 3, paragraphes,
/// listes a puces et numerotees, citations, blocs de code encadres,
/// separateurs, et en ligne le gras, l'italique, le code et les liens.
/// De quoi ecrire une annonce de coupe, ce qui est le besoin.
///
/// **Les images deviennent des liens.** La politique de securite du
/// contenu de Cairn n'autorise aucune image distante, donc une image
/// d'article ne s'afficherait pas : elle resterait un cadre vide. Un lien
/// nomme dit au moins ce qu'il y a au bout.
pub fn analyser(corps: &str) -> Vec<Bloc> {
    let mut blocs = Vec::new();
    let lignes: Vec<&str> = corps.lines().map(|l| l.trim_end()).collect();
    let mut i = 0;
    while i < lignes.len() {
        let ligne = lignes[i];
        let nu = ligne.trim();

        if nu.is_empty() {
            i += 1;
            continue;
        }

        // Bloc de code encadre.
        if nu.starts_with("```") {
            let mut contenu = Vec::new();
            i += 1;
            while i < lignes.len() && !lignes[i].trim().starts_with("```") {
                contenu.push(lignes[i]);
                i += 1;
            }
            // Une cloture manquante ne doit pas perdre le texte : on prend
            // ce qu'on a jusqu'au bout.
            if i < lignes.len() {
                i += 1;
            }
            blocs.push(Bloc::Code {
                texte: contenu.join("\n"),
            });
            continue;
        }

        if nu == "---" || nu == "***" || nu == "___" {
            blocs.push(Bloc::Separateur);
            i += 1;
            continue;
        }

        if let Some(niveau) = niveau_de_titre(nu) {
            let texte = nu[niveau as usize..].trim().to_string();
            blocs.push(Bloc::Titre { niveau, texte });
            i += 1;
            continue;
        }

        if nu.starts_with("> ") || nu == ">" {
            let mut morceaux = Vec::new();
            while i < lignes.len() {
                let l = lignes[i].trim();
                if !(l.starts_with("> ") || l == ">") {
                    break;
                }
                morceaux.push(l.trim_start_matches('>').trim());
                i += 1;
            }
            blocs.push(Bloc::Citation {
                fragments: en_ligne(&morceaux.join(" ")),
            });
            continue;
        }

        if let Some(ordonnee) = sorte_de_liste(nu) {
            let mut points = Vec::new();
            while i < lignes.len() {
                let l = lignes[i].trim();
                match sorte_de_liste(l) {
                    Some(meme) if meme == ordonnee => {
                        points.push(en_ligne(contenu_de_point(l)));
                        i += 1;
                    }
                    _ => break,
                }
            }
            blocs.push(Bloc::Liste { ordonnee, points });
            continue;
        }

        // Un paragraphe court jusqu'a la ligne vide, et ses retours a la
        // ligne deviennent des espaces, comme en markdown.
        let mut morceaux = Vec::new();
        while i < lignes.len() {
            let l = lignes[i].trim();
            if l.is_empty()
                || l.starts_with("```")
                || niveau_de_titre(l).is_some()
                || sorte_de_liste(l).is_some()
                || l.starts_with("> ")
                || l == "---"
            {
                break;
            }
            morceaux.push(l);
            i += 1;
        }
        blocs.push(Bloc::Paragraphe {
            fragments: en_ligne(&morceaux.join(" ")),
        });
    }
    blocs
}

fn niveau_de_titre(ligne: &str) -> Option<u8> {
    for niveau in (1u8..=3).rev() {
        let prefixe = "#".repeat(niveau as usize) + " ";
        if ligne.starts_with(&prefixe) {
            return Some(niveau);
        }
    }
    None
}

/// `Some(false)` pour une puce, `Some(true)` pour une numerotee.
fn sorte_de_liste(ligne: &str) -> Option<bool> {
    if ligne.starts_with("- ") || ligne.starts_with("* ") {
        return Some(false);
    }
    let chiffres: String = ligne.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !chiffres.is_empty() && ligne[chiffres.len()..].starts_with(". ") {
        return Some(true);
    }
    None
}

fn contenu_de_point(ligne: &str) -> &str {
    if ligne.starts_with("- ") || ligne.starts_with("* ") {
        return ligne[2..].trim();
    }
    let chiffres = ligne.chars().take_while(|c| c.is_ascii_digit()).count();
    ligne[chiffres + 2..].trim()
}

/// Decoupe le gras, l'italique, le code et les liens d'une ligne.
///
/// Une marque qui ne se referme pas reste du texte ordinaire. C'est le
/// comportement qu'on attend : un article ou quelqu'un a tape une
/// etoile ne doit pas voir la moitie de sa phrase disparaitre.
pub fn en_ligne(ligne: &str) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let mut tampon = String::new();
    let c: Vec<char> = ligne.chars().collect();
    let mut i = 0;

    let poser = |tampon: &mut String, fragments: &mut Vec<Fragment>| {
        if !tampon.is_empty() {
            fragments.push(Fragment::Texte {
                texte: std::mem::take(tampon),
            });
        }
    };

    while i < c.len() {
        // Image, qui doit etre essayee avant le lien a cause du `!`.
        if c[i] == '!' && i + 1 < c.len() && c[i + 1] == '[' {
            if let Some((texte, lien, fin)) = lien_a(&c, i + 1) {
                poser(&mut tampon, &mut fragments);
                fragments.push(if lien_autorise(&lien) {
                    Fragment::Image { texte, lien }
                } else {
                    Fragment::Texte { texte }
                });
                i = fin;
                continue;
            }
        }
        if c[i] == '[' {
            if let Some((texte, lien, fin)) = lien_a(&c, i) {
                poser(&mut tampon, &mut fragments);
                // Un lien refuse garde son libelle et perd sa destination.
                // Le lecteur voit le texte, il ne peut simplement pas
                // cliquer vers autre chose que le web.
                fragments.push(if lien_autorise(&lien) {
                    Fragment::Lien { texte, lien }
                } else {
                    Fragment::Texte { texte }
                });
                i = fin;
                continue;
            }
        }
        if c[i] == '`' {
            if let Some(fin) = jusqu_a(&c, i + 1, '`') {
                poser(&mut tampon, &mut fragments);
                fragments.push(Fragment::Code {
                    texte: c[i + 1..fin].iter().collect(),
                });
                i = fin + 1;
                continue;
            }
        }
        if c[i] == '*' && i + 1 < c.len() && c[i + 1] == '*' {
            if let Some(fin) = double_etoile(&c, i + 2) {
                poser(&mut tampon, &mut fragments);
                fragments.push(Fragment::Gras {
                    texte: c[i + 2..fin].iter().collect(),
                });
                i = fin + 2;
                continue;
            }
        }
        if c[i] == '*' {
            if let Some(fin) = jusqu_a(&c, i + 1, '*') {
                poser(&mut tampon, &mut fragments);
                fragments.push(Fragment::Italique {
                    texte: c[i + 1..fin].iter().collect(),
                });
                i = fin + 1;
                continue;
            }
        }
        tampon.push(c[i]);
        i += 1;
    }
    poser(&mut tampon, &mut fragments);
    fragments
}

fn jusqu_a(c: &[char], depuis: usize, marque: char) -> Option<usize> {
    let mut i = depuis;
    while i < c.len() {
        if c[i] == marque {
            // Une marque collee a celle d'ouverture donne un contenu vide,
            // qui n'est pas une mise en forme.
            return if i > depuis { Some(i) } else { None };
        }
        i += 1;
    }
    None
}

fn double_etoile(c: &[char], depuis: usize) -> Option<usize> {
    let mut i = depuis;
    while i + 1 < c.len() {
        if c[i] == '*' && c[i + 1] == '*' {
            return if i > depuis { Some(i) } else { None };
        }
        i += 1;
    }
    None
}

/// Lit `[texte](lien)` a partir de `[`. Rend le texte, le lien, et l'index
/// qui suit la parenthese fermante.
fn lien_a(c: &[char], ouvrante: usize) -> Option<(String, String, usize)> {
    let fin_texte = jusqu_a(c, ouvrante + 1, ']')?;
    if fin_texte + 1 >= c.len() || c[fin_texte + 1] != '(' {
        return None;
    }
    let fin_lien = jusqu_a(c, fin_texte + 2, ')')?;
    Some((
        c[ouvrante + 1..fin_texte].iter().collect(),
        c[fin_texte + 2..fin_lien].iter().collect(),
        fin_lien + 1,
    ))
}

// -------------------------------------------------------- mise en forme

/// Prepare les articles d'un fil pour l'affichage.
///
/// Les epingles passent devant, puis les plus recents. Un article dans
/// une autre langue que celle lue est ecarte, sauf s'il n'a pas de
/// langue, auquel cas il vaut pour les deux.
pub fn preparer(fil: &Fil, langue: &str) -> Vec<ArticleRendu> {
    let mut rendus: Vec<ArticleRendu> = fil
        .articles
        .iter()
        .filter(|a| a.langue.is_empty() || a.langue.eq_ignore_ascii_case(langue))
        .map(|a| ArticleRendu {
            id: a.id.clone(),
            titre: a.titre.clone(),
            categorie: Categorie::depuis_cle(&a.categorie),
            date: a.date,
            resume: a.resume.clone(),
            langue: a.langue.clone(),
            epingle: a.epingle,
            corps: analyser(&a.corps),
        })
        .collect();
    rendus.sort_by(|a, b| b.epingle.cmp(&a.epingle).then(b.date.cmp(&a.date)));
    rendus
}

/// Compte les articles par categorie, pour les filtres comptes.
pub fn compter(articles: &[ArticleRendu]) -> Vec<(Categorie, usize)> {
    Categorie::toutes()
        .iter()
        .map(|c| (*c, articles.iter().filter(|a| a.categorie == *c).count()))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// Filtre par categories et par recherche.
///
/// La recherche est celle de la liste des serveurs et du journal, voir
/// `recherche::correspond`. Trois barres de recherche dans la meme
/// application doivent se comporter pareil.
///
/// **Une liste de categories vide ne filtre rien**, contrairement au
/// journal ou decocher tout vide la liste. La difference est voulue :
/// ici les categories ne sont pas cochees au depart, elles s'ajoutent, et
/// « aucune choisie » veut dire « toutes » et non « rien ».
pub fn filtrer<'a>(
    articles: &'a [ArticleRendu],
    categories: &[Categorie],
    recherche: &str,
) -> Vec<&'a ArticleRendu> {
    articles
        .iter()
        .filter(|a| categories.is_empty() || categories.contains(&a.categorie))
        .filter(|a| crate::recherche::correspond(&[&a.titre, &a.resume], recherche))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------- signature

    /// Un trousseau fixe, fabrique une fois et ecrit ici.
    ///
    /// La cle privee n'a aucune valeur : elle ne sert qu'a ces tests et
    /// n'a jamais signe quoi que ce soit de publie.
    const CLE_PUBLIQUE: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const MESSAGE: &[u8] = b"";
    const SIGNATURE: &str = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";

    #[test]
    fn une_signature_juste_est_acceptee() {
        // Vecteur d'essai 1 de la RFC 8032, section 7.1.
        assert!(signature_valable(MESSAGE, SIGNATURE, CLE_PUBLIQUE));
    }

    #[test]
    fn un_message_modifie_est_refuse() {
        assert!(!signature_valable(b"x", SIGNATURE, CLE_PUBLIQUE));
    }

    #[test]
    fn une_signature_modifiee_est_refusee() {
        let mut abimee = SIGNATURE.to_string();
        abimee.replace_range(0..1, "f");
        assert!(!signature_valable(MESSAGE, &abimee, CLE_PUBLIQUE));
    }

    #[test]
    fn une_autre_cle_est_refusee() {
        let autre = "0".repeat(64);
        assert!(!signature_valable(MESSAGE, SIGNATURE, &autre));
    }

    #[test]
    fn une_signature_mal_formee_est_refusee_sans_paniquer() {
        for mauvaise in [
            "",
            "zz",
            "nonhexadecimal",
            &"a".repeat(127),
            &"a".repeat(129),
        ] {
            assert!(
                !signature_valable(MESSAGE, mauvaise, CLE_PUBLIQUE),
                "{mauvaise}"
            );
        }
        for mauvaise_cle in ["", "abc", &"g".repeat(64)] {
            assert!(!signature_valable(MESSAGE, SIGNATURE, mauvaise_cle));
        }
    }

    #[test]
    fn l_echantillon_livre_est_signe_par_la_cle_compilee() {
        // Le controle de bout en bout de la chaine de publication :
        // l'outil `examples/actualites.rs` a signe ce fichier, et la cle
        // de `lib.rs` le verifie. Si quelqu'un remplace la cle sans
        // resigner l'echantillon, ou l'inverse, ce test le dit.
        let octets = include_bytes!("../../../docs/actualites/actualites.json");
        let signature = include_str!("../../../docs/actualites/actualites.json.sig");
        assert!(signature_valable(octets, signature, crate::CLE_ACTUALITES));

        // Et il se lit vraiment, avec des articles dedans.
        let fil: Fil = serde_json::from_slice(octets).unwrap();
        assert_eq!(fil.version, VERSION_FIL);
        assert!(fil.articles.len() >= 3);
        assert!(!preparer(&fil, "fr").is_empty());
        assert!(!preparer(&fil, "en").is_empty());
    }

    #[test]
    fn un_octet_change_dans_l_echantillon_casse_la_signature() {
        let mut octets = include_bytes!("../../../docs/actualites/actualites.json").to_vec();
        let signature = include_str!("../../../docs/actualites/actualites.json.sig");
        let dernier = octets.len() - 1;
        octets[dernier] = b' ';
        assert!(!signature_valable(
            &octets,
            signature,
            crate::CLE_ACTUALITES
        ));
    }

    #[test]
    fn l_emplacement_reserve_est_reconnu_comme_tel() {
        // Tel qu'il est livre, le fil n'est pas configure : l'adresse est
        // en `.invalid`, un domaine que la RFC 2606 rend impossible a
        // enregistrer. Le jour ou quelqu'un met une vraie adresse, ce
        // test tombe, et c'est exactement ce qu'on veut : il rappelle de
        // verifier que le controle suit.
        assert!(!crate::actualites_configurees());
        assert!(crate::URL_ACTUALITES.contains(".invalid/"));
        assert!(crate::URL_ACTUALITES_SIGNATURE.contains(".invalid/"));
    }

    #[test]
    fn les_cles_de_souci_sont_toutes_distinctes() {
        // Deux soucis qui rendraient la meme cle donneraient la meme
        // phrase a l'ecran, et l'un des deux serait invisible.
        let tous = [
            Souci::Reseau,
            Souci::Signature,
            Souci::Illisible,
            Souci::Rejeu,
            Souci::TropGros,
            Souci::NonConfigure,
        ];
        let mut cles: Vec<&str> = tous.iter().map(|s| s.cle()).collect();
        cles.sort_unstable();
        let avant = cles.len();
        cles.dedup();
        assert_eq!(cles.len(), avant);
    }

    #[test]
    fn le_rejeu_d_un_ancien_fil_est_refuse() {
        assert!(plus_recent_que(100, None));
        assert!(plus_recent_que(100, Some(100)));
        assert!(plus_recent_que(101, Some(100)));
        assert!(!plus_recent_que(99, Some(100)));
    }

    // ------------------------------------------------------------- liens

    #[test]
    fn seuls_le_web_passe() {
        assert!(lien_autorise("https://shootmania.exchange/maps/1234"));
        assert!(lien_autorise("http://example.org"));
        for refuse in [
            "file:///C:/Windows/System32/cmd.exe",
            "\\\\serveur\\partage",
            "javascript:alert(1)",
            "data:text/html,<script>",
            "maniaplanet://join/xyz",
            "ftp://example.org",
            "https://",
            "",
        ] {
            assert!(!lien_autorise(refuse), "{refuse}");
        }
    }

    #[test]
    fn un_hote_camoufle_est_refuse() {
        // La vraie destination est `mechant.fr`, pas `nadeo.com`.
        assert!(!lien_autorise("https://nadeo.com@mechant.fr/page"));
    }

    #[test]
    fn les_caracteres_de_controle_sont_refuses() {
        assert!(!lien_autorise("https://exa\nmple.org"));
        assert!(!lien_autorise("https://example.org/\u{0}"));
        assert!(!lien_autorise("https://example.org/\"onclick=x"));
    }

    // ---------------------------------------------------------- markdown

    #[test]
    fn les_titres_et_les_paragraphes() {
        let blocs = analyser("# Grand\n\nUn texte.\n\n## Moyen");
        assert_eq!(
            blocs[0],
            Bloc::Titre {
                niveau: 1,
                texte: "Grand".into()
            }
        );
        assert_eq!(
            blocs[1],
            Bloc::Paragraphe {
                fragments: vec![Fragment::Texte {
                    texte: "Un texte.".into()
                }]
            }
        );
        assert_eq!(
            blocs[2],
            Bloc::Titre {
                niveau: 2,
                texte: "Moyen".into()
            }
        );
    }

    #[test]
    fn un_paragraphe_recolle_ses_lignes() {
        let blocs = analyser("Une phrase\ncoupee en deux.");
        assert_eq!(
            blocs,
            vec![Bloc::Paragraphe {
                fragments: vec![Fragment::Texte {
                    texte: "Une phrase coupee en deux.".into()
                }]
            }]
        );
    }

    #[test]
    fn les_deux_sortes_de_listes() {
        let blocs = analyser("- un\n- deux\n\n1. a\n2. b");
        match &blocs[0] {
            Bloc::Liste { ordonnee, points } => {
                assert!(!ordonnee);
                assert_eq!(points.len(), 2);
            }
            autre => panic!("{autre:?}"),
        }
        match &blocs[1] {
            Bloc::Liste { ordonnee, points } => {
                assert!(ordonnee);
                assert_eq!(points.len(), 2);
            }
            autre => panic!("{autre:?}"),
        }
    }

    #[test]
    fn le_bloc_de_code_garde_ses_lignes() {
        let blocs = analyser("```\nune\ndeux\n```");
        assert_eq!(
            blocs,
            vec![Bloc::Code {
                texte: "une\ndeux".into()
            }]
        );
    }

    #[test]
    fn un_bloc_de_code_non_ferme_ne_perd_rien() {
        let blocs = analyser("```\nune\ndeux");
        assert_eq!(
            blocs,
            vec![Bloc::Code {
                texte: "une\ndeux".into()
            }]
        );
    }

    #[test]
    fn le_contenu_d_un_bloc_de_code_n_est_pas_interprete() {
        // Sinon un `# ` dans du code deviendrait un titre.
        let blocs = analyser("```\n# pas un titre\n- pas une liste\n```");
        assert_eq!(
            blocs,
            vec![Bloc::Code {
                texte: "# pas un titre\n- pas une liste".into()
            }]
        );
    }

    #[test]
    fn la_mise_en_forme_en_ligne() {
        let f = en_ligne("du **gras**, de l'*italique* et du `code`");
        assert!(f.contains(&Fragment::Gras {
            texte: "gras".into()
        }));
        assert!(f.contains(&Fragment::Italique {
            texte: "italique".into()
        }));
        assert!(f.contains(&Fragment::Code {
            texte: "code".into()
        }));
    }

    #[test]
    fn une_marque_non_fermee_reste_du_texte() {
        // Le controle qui compte : un article ou quelqu'un a tape une
        // etoile ne doit pas voir la moitie de sa phrase disparaitre.
        let f = en_ligne("une etoile * toute seule");
        assert_eq!(
            f,
            vec![Fragment::Texte {
                texte: "une etoile * toute seule".into()
            }]
        );
        let f = en_ligne("du **gras pas ferme");
        assert_eq!(
            f,
            vec![Fragment::Texte {
                texte: "du **gras pas ferme".into()
            }]
        );
    }

    #[test]
    fn un_lien_est_reconnu() {
        let f = en_ligne("voir [la map](https://sm.exchange/m/1) ici");
        assert!(f.contains(&Fragment::Lien {
            texte: "la map".into(),
            lien: "https://sm.exchange/m/1".into()
        }));
    }

    #[test]
    fn un_lien_interdit_garde_son_texte_et_perd_sa_destination() {
        // Le controle qui compte. Un article est ecrit par quelqu'un
        // d'autre, et ce lien part dans un appel systeme.
        let f = en_ligne("clique [ici](file:///C:/Windows/System32/cmd.exe)");
        assert!(f.contains(&Fragment::Texte {
            texte: "ici".into()
        }));
        assert!(!f.iter().any(|x| matches!(x, Fragment::Lien { .. })));
    }

    #[test]
    fn une_image_devient_un_lien_nomme() {
        let f = en_ligne("![la carte](https://exemple.org/a.png)");
        assert_eq!(
            f,
            vec![Fragment::Image {
                texte: "la carte".into(),
                lien: "https://exemple.org/a.png".into()
            }]
        );
    }

    #[test]
    fn aucun_fragment_ne_porte_de_html() {
        // Il n'y a pas d'echappement a faire parce qu'il n'y a pas de
        // HTML : ce qui ressemble a une balise reste du texte, et c'est
        // Svelte qui l'ecrira comme un noeud de texte.
        let f = en_ligne("<script>alert(1)</script>");
        assert_eq!(
            f,
            vec![Fragment::Texte {
                texte: "<script>alert(1)</script>".into()
            }]
        );
    }

    // ------------------------------------------------------- le fil entier

    fn fil_exemple() -> Fil {
        serde_json::from_str(
            r#"{
              "version": 1,
              "publie": 1790000000,
              "articles": [
                {"id":"a","titre":"Coupe Atria","categorie":"tournoi","date":300,
                 "resume":"Samedi 21h","corps":"Une **coupe**.","langue":"fr"},
                {"id":"b","titre":"Frostline","categorie":"maps","date":400,
                 "resume":"Nouveau mappack","corps":"Des maps.","langue":""},
                {"id":"c","titre":"Serveur","categorie":"inventee","date":100,
                 "resume":"","corps":"","langue":"fr","epingle":true},
                {"id":"d","titre":"Only english","categorie":"maps","date":500,
                 "resume":"","corps":"","langue":"en"}
              ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn une_categorie_inconnue_ne_fait_pas_perdre_l_article() {
        // La lecon du numero de contrat, appliquee aux donnees : une
        // publication plus recente ne doit pas couper les actualites de
        // ceux qui n'ont pas mis a jour.
        let rendus = preparer(&fil_exemple(), "fr");
        let c = rendus.iter().find(|a| a.id == "c").unwrap();
        assert_eq!(c.categorie, Categorie::Autre);
    }

    #[test]
    fn les_epingles_passent_devant_puis_les_recents() {
        let rendus = preparer(&fil_exemple(), "fr");
        let ordre: Vec<&str> = rendus.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ordre, vec!["c", "b", "a"]);
    }

    #[test]
    fn la_langue_ecarte_ce_qui_ne_la_concerne_pas() {
        let fr = preparer(&fil_exemple(), "fr");
        assert!(!fr.iter().any(|a| a.id == "d"));
        let en = preparer(&fil_exemple(), "en");
        assert!(en.iter().any(|a| a.id == "d"));
        // Un article sans langue vaut dans les deux.
        assert!(fr.iter().any(|a| a.id == "b"));
        assert!(en.iter().any(|a| a.id == "b"));
    }

    #[test]
    fn un_champ_absent_ne_fait_pas_echouer_la_lecture() {
        let fil: Fil = serde_json::from_str(
            r#"{"version":1,"publie":1,"articles":[{"id":"x","titre":"T","date":5}]}"#,
        )
        .unwrap();
        let rendus = preparer(&fil, "fr");
        assert_eq!(rendus.len(), 1);
        assert_eq!(rendus[0].categorie, Categorie::Autre);
    }

    #[test]
    fn les_comptes_et_le_filtre() {
        let rendus = preparer(&fil_exemple(), "fr");
        let comptes = compter(&rendus);
        assert!(comptes.contains(&(Categorie::Tournoi, 1)));
        assert!(comptes.contains(&(Categorie::Maps, 1)));

        // Aucune categorie choisie veut dire toutes.
        assert_eq!(filtrer(&rendus, &[], "").len(), 3);
        assert_eq!(filtrer(&rendus, &[Categorie::Tournoi], "").len(), 1);
        assert_eq!(filtrer(&rendus, &[], "frostline").len(), 1);
        assert_eq!(filtrer(&rendus, &[], "FROSTLINE").len(), 1);
    }

    #[test]
    fn le_corps_est_decoupe_a_la_preparation() {
        let rendus = preparer(&fil_exemple(), "fr");
        let a = rendus.iter().find(|x| x.id == "a").unwrap();
        assert_eq!(
            a.corps,
            vec![Bloc::Paragraphe {
                fragments: vec![
                    Fragment::Texte {
                        texte: "Une ".into()
                    },
                    Fragment::Gras {
                        texte: "coupe".into()
                    },
                    Fragment::Texte { texte: ".".into() },
                ]
            }]
        );
    }
}
