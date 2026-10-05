//! Cairn, coquille de l'application.
//!
//! Cette couche ne contient aucune logique metier. Elle expose des commandes
//! a l'interface et delegue tout le reste a `cairn-core`, qui est testable
//! sans fenetre et sans reseau. C'est la regle a tenir : si une fonction
//! merite un test, elle n'a rien a faire ici.
//!
//! Les quatre commandes reprennent le contrat de la passerelle PowerShell du
//! lot 18, deja eprouve par l'interface :
//!
//! | Commande    | Ce qu'elle fait |
//! |-------------|-----------------|
//! | `diagnostic`| versions et systeme, sert a verifier le pont |
//! | `etat`      | chemin du jeu, profils, jeu lance ou non |
//! | `veille`    | le coup d'oeil des cinq secondes, voir `cairn_core::veille` |
//! | `serveurs`  | les serveurs ShootMania en ligne |
//! | `vue_serveurs` | cette liste, filtree, triee et groupee |
//! | `tri_suivant` | ce que devient le tri quand on clique sur un critere |
//! | `naviguer`  | va sur un ecran et l'empile dans l'historique |
//! | `reculer`   | revient a l'ecran precedent |
//! | `avancer`   | repart vers l'avant, apres un retour |
//! | `basculer_favori` | met ou enleve un serveur des favoris |
//! | `rejoindre` | lance le jeu sur un serveur |
//! | `ouvrir_jeu`| ouvre le jeu sur ses propres menus |
//! | `quitter`   | ferme l'application |
//! | `fermer_ou_reduire` | la reponse a la question posee par la croix |
//! | `glisser_tuile` | une tuile portee au dessus d'une autre |
//! | `inventaire` | les maps, replays et title packs du joueur |
//! | `ouvrir_dossier` | ouvre l'un des trois dossiers du jeu |
//! | `ouvrir_titre` | ouvre le jeu sur le menu d'un title pack |
//! | `decaler_tuile` | une tuile poussee d'un cran |
//!
//! Le lot 19c y ajoute les reglages, que l'interface lit au demarrage et
//! reecrit a chaque changement.
//!
//! | Commande               | Ce qu'elle fait |
//! |------------------------|-----------------|
//! | `reglages`             | ce que Cairn a retenu du dernier lancement |
//! | `enregistrer_reglages` | ecrit ces reglages sur le disque |

mod actualites;
mod api;
mod demarrage;
mod jeu;
mod journal;
mod plateforme;
mod stockage;

use cairn_core::reglages::Reglages;

/// L'ecran sur lequel Cairn s'ouvre, et donc la premiere entree de
/// l'historique. Le meme identifiant que celui de l'interface.
const ECRAN_DEPART: &str = "play";
use cairn_core::Plateforme;
use std::sync::Mutex;

/// Les reglages tels qu'ils sont a cet instant.
///
/// Ils vivent en memoire et pas seulement dans le fichier : `etat` est
/// appele toutes les cinq secondes et a besoin du chemin du jeu designe a
/// la main. Relire un fichier a chaque fois serait une lecture disque
/// toutes les cinq secondes pour rien.
struct Memoire {
    reglages: Mutex<Reglages>,
    /// La derniere liste de serveurs rapportee du reseau.
    ///
    /// Elle est gardee ici pour que le filtrage, le tri et la recherche
    /// de l'ecran Parcourir les serveurs se fassent dans `cairn-core`,
    /// ou ils sont testes, sans que l'interface ait a renvoyer toute la
    /// liste a chaque frappe.
    serveurs: Mutex<Vec<cairn_core::serveurs::Serveur>>,
    /// Le chemin du jeu trouve au dernier releve complet, et l'empreinte
    /// du dossier des profils au dernier coup d'oeil. Voir
    /// [`cairn_core::veille`] : c'est ce qui permet a la boucle de cinq
    /// secondes de ne presque rien couter.
    chemin_jeu: Mutex<Option<String>>,
    empreinte: Mutex<Option<cairn_core::veille::Empreinte>>,
    /// L'historique de navigation, voir [`cairn_core::historique`]. Il
    /// vit ici et pas dans l'interface parce que c'est une regle, avec
    /// ses cas limites, et que les regles se testent.
    historique: Mutex<cairn_core::historique::Historique>,
}

impl Memoire {
    /// Une copie de travail. Le verrou n'est jamais tenu pendant une
    /// entree-sortie, seulement le temps de cloner.
    fn lire(&self) -> Reglages {
        verrou(&self.reglages).clone()
    }

    fn ecrire(&self, valeurs: Reglages) {
        *verrou(&self.reglages) = valeurs;
    }
}

/// Prend un verrou, empoisonne ou non.
///
/// Un verrou empoisonne veut dire qu'un fil a panique en le tenant. La
/// donnee derriere reste lisible et coherente, donc on continue avec,
/// plutot que de faire tomber l'application pour une histoire de fil.
fn verrou<T>(cadenas: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    cadenas.lock().unwrap_or_else(|e| e.into_inner())
}

/// Versions et systeme.
///
/// Elle sert a deux choses : verifier que le pont fonctionne, et porter
/// le numero de contrat que l'interface compare au sien au demarrage.
#[tauri::command]
fn diagnostic() -> serde_json::Value {
    serde_json::json!({
        "nom": "Cairn",
        "version": env!("CARGO_PKG_VERSION"),
        "coeur": cairn_core::VERSION,
        // Le numero de contrat du pont, voir cairn_core::CONTRAT.
        // L'interface le compare a celui qu'elle attend et previent
        // quand les deux moities ne sont pas de la meme version.
        "contrat": cairn_core::CONTRAT,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "journal": journal::chemin().map(|c| c.display().to_string()),
    })
}

/// Ce que Cairn sait de l'installation.
///
/// `profil_memorise` est l'identifiant que l'interface avait retenu. Il est
/// recoupe avec les profils qui existent vraiment, parce qu'un profil
/// renomme ou supprime dans le jeu ferait echouer le lancement sans le
/// moindre message.
#[tauri::command]
async fn etat(
    profil_memorise: Option<String>,
    memoire: tauri::State<'_, Memoire>,
) -> Result<cairn_core::Etat, String> {
    let reglages = memoire.lire();
    let connu = verrou(&memoire.chemin_jeu).clone();
    // Registre, disque et liste des processus : c'est bloquant, donc on ne
    // le fait pas sur le fil qui dessine la fenetre.
    let (etat, empreinte) = tauri::async_runtime::spawn_blocking(move || {
        let systeme = plateforme::Systeme::depuis_avec_cache(&reglages, connu);
        let etat = cairn_core::etat(&systeme, profil_memorise.as_deref());
        let empreinte = cairn_core::veille::Empreinte {
            jeu_lance: etat.jeu_lance,
            profils: plateforme::empreinte_profils(),
        };
        (etat, empreinte)
    })
    .await
    .map_err(|e| {
        journal::alerte(&format!("Releve d'etat interrompu : {e}"));
        "Cairn n'a pas pu lire l'etat de l'installation.".to_string()
    })?;

    // Le releve complet fait aussi office de coup d'oeil : sans ca, le
    // suivant croirait que tout vient de changer.
    *verrou(&memoire.chemin_jeu) = etat.chemin_jeu.clone();
    *verrou(&memoire.empreinte) = Some(empreinte);
    Ok(etat)
}

/// Le coup d'oeil des cinq secondes.
///
/// Il ne rapporte que ce qui bouge vraiment : le jeu est-il ouvert, et
/// le dossier des profils a-t-il change. Pas de registre, pas de lecture
/// de profils, pas de ligne de journal tant que rien ne change. C'est le
/// lot 19d, ne d'une remarque de Flow : un launcher qui interroge le
/// systeme en boucle prepare des lags et des ennuis.
#[tauri::command]
async fn veille(memoire: tauri::State<'_, Memoire>) -> Result<cairn_core::veille::Veille, String> {
    let avant = verrou(&memoire.empreinte).clone();
    let maintenant = tauri::async_runtime::spawn_blocking(|| cairn_core::veille::Empreinte {
        jeu_lance: plateforme::jeu_lance(),
        profils: plateforme::empreinte_profils(),
    })
    .await
    .map_err(|e| {
        journal::alerte(&format!("Coup d'oeil interrompu : {e}"));
        "Cairn n'a pas pu regarder l'etat du jeu.".to_string()
    })?;

    let veille = cairn_core::veille::comparer(avant.as_ref(), &maintenant);
    *verrou(&memoire.empreinte) = Some(maintenant);
    if veille.jeu_a_change {
        journal::ecrire(if veille.jeu_lance {
            "Le jeu vient de s'ouvrir."
        } else {
            "Le jeu vient de se fermer."
        });
    }
    Ok(veille)
}

/// Les serveurs ShootMania en ligne, deja tries et nettoyes.
///
/// La liste est aussi gardee en memoire : c'est elle que [`vue_serveurs`]
/// filtre ensuite, sans nouvel appel reseau.
#[tauri::command]
async fn serveurs(
    memoire: tauri::State<'_, Memoire>,
) -> Result<Vec<cairn_core::serveurs::Serveur>, String> {
    let liste = api::serveurs_en_ligne().await?;
    *verrou(&memoire.serveurs) = liste.clone();
    Ok(liste)
}

/// L'ecran Parcourir les serveurs, filtre et trie.
///
/// Travaille sur la liste deja rapportee, donc sans reseau : c'est
/// appele a chaque frappe dans la recherche. Toute la logique est dans
/// `cairn_core::serveurs::vue`, ou elle est testee.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn vue_serveurs(
    mode: Option<String>,
    recherche: String,
    tri: cairn_core::serveurs::Tri,
    croissant: bool,
    mot_de_passe: String,
    favoris_seuls: bool,
    memoire: tauri::State<'_, Memoire>,
) -> cairn_core::serveurs::Vue {
    let liste = verrou(&memoire.serveurs).clone();
    // Les favoris ne font pas l'aller-retour a chaque frappe : ils sont
    // deja en memoire, et c'est la commande `basculer_favori` qui les
    // tient a jour.
    let reglages = memoire.lire();
    let prive = cairn_core::reglages::prive_demande(&mot_de_passe);
    let filtre = cairn_core::serveurs::Filtre {
        mode: mode.as_deref().filter(|m| !m.is_empty()),
        recherche: &recherche,
        prive,
        favoris_seuls,
        favoris: &reglages.favoris,
    };
    cairn_core::serveurs::vue(&liste, &filtre, tri, croissant)
}

/// Ce que devient le tri quand on clique sur un critere.
///
/// Une commande pour trois lignes, et c'est voulu : le cycle a trois
/// etats est une regle, elle vit dans `cairn-core` ou elle est testee, et
/// la reecrire en TypeScript pour s'epargner un aller-retour local serait
/// exactement le genre de duplication qui finit par diverger.
#[tauri::command]
fn tri_suivant(
    courant: cairn_core::serveurs::Tri,
    croissant: bool,
    demande: cairn_core::serveurs::Tri,
) -> (cairn_core::serveurs::Tri, bool) {
    cairn_core::serveurs::tri_suivant(courant, croissant, demande)
}

/// Va sur un ecran, et l'empile dans l'historique.
///
/// Toutes les navigations de l'interface passent par ici, d'ou qu'elles
/// viennent : un onglet clique, les deux touches, la roue dentee, une
/// tuile. C'est ce qui permet a Precedent de remonter le chemin
/// reellement parcouru et non la rangee d'onglets.
#[tauri::command]
fn naviguer(
    ecran: String,
    memoire: tauri::State<'_, Memoire>,
) -> cairn_core::historique::EtatNavigation {
    verrou(&memoire.historique).naviguer(&ecran)
}

/// Revient a l'ecran precedent.
#[tauri::command]
fn reculer(memoire: tauri::State<'_, Memoire>) -> cairn_core::historique::EtatNavigation {
    verrou(&memoire.historique).reculer()
}

/// Repart vers l'avant, apres un retour.
#[tauri::command]
fn avancer(memoire: tauri::State<'_, Memoire>) -> cairn_core::historique::EtatNavigation {
    verrou(&memoire.historique).avancer()
}

/// Une tuile portee au dessus d'une autre pendant un glisser.
///
/// Appelee a chaque fois que le pointeur entre dans une autre tuile, et
/// pas une seule fois au lacher : la grille se reorganise sous le
/// curseur, donc l'ordre affiche est deja l'ordre final. Les reglages ne
/// sont ecrits qu'au lacher, l'interface s'en charge.
///
/// La regle vit dans `cairn-core`, ou elle est testee, pour la meme
/// raison que sa voisine : le retrait avant l'insertion decale d'un cran
/// toutes les places situees apres l'ancienne.
///
/// `grille` dit laquelle des deux grilles on range, Jouer ou Editer.
/// Une cle inconnue rend l'ordre inchange plutot que de deviner : les
/// deux grilles n'ont aucune tuile en commun, se tromper reviendrait a
/// toutes les jeter.
#[tauri::command]
fn glisser_tuile(grille: String, ordre: Vec<String>, depuis: String, vers: String) -> Vec<String> {
    match cairn_core::reglages::Grille::depuis_cle(&grille) {
        Some(grille) => cairn_core::reglages::glisser_sur(grille, &ordre, &depuis, &vers),
        None => ordre,
    }
}

/// Decale une tuile d'un cran, pour les fleches et le clavier.
#[tauri::command]
fn decaler_tuile(grille: String, ordre: Vec<String>, id: String, pas: i32) -> Vec<String> {
    match cairn_core::reglages::Grille::depuis_cle(&grille) {
        Some(grille) => cairn_core::reglages::decaler_tuile(grille, &ordre, &id, pas),
        None => ordre,
    }
}

/// L'inventaire des fichiers du joueur, pour l'ecran Editer.
///
/// Trois lectures de dossier, sans ouvrir le moindre fichier. Elle n'est
/// appelee qu'a l'ouverture de l'ecran et sur demande expresse, pas dans
/// la boucle de veille : parcourir un dossier `Downloaded` qui compte des
/// milliers de maps toutes les cinq secondes serait exactement le defaut
/// que le lot 19d a corrige ailleurs.
#[tauri::command]
async fn inventaire() -> std::collections::HashMap<String, cairn_core::inventaire::Inventaire> {
    use cairn_core::inventaire::Sorte;
    tauri::async_runtime::spawn_blocking(|| {
        [Sorte::Maps, Sorte::Replays, Sorte::Packs]
            .into_iter()
            .map(|sorte| (sorte.cle().to_string(), plateforme::inventorier(sorte)))
            .collect()
    })
    .await
    .unwrap_or_default()
}

/// Ouvre l'un des trois dossiers du jeu dans l'explorateur.
///
/// **L'interface envoie une cle et jamais un chemin.** Aucun chemin ne
/// traverse donc le pont, il n'y a rien a valider et rien a contourner :
/// les seuls dossiers qui puissent s'ouvrir sont les trois que Cairn
/// connait.
#[tauri::command]
fn ouvrir_dossier(cle: String) -> Result<(), String> {
    let Some(sorte) = cairn_core::inventaire::Sorte::depuis_cle(&cle) else {
        return Err("Cairn ne connait pas ce dossier.".to_string());
    };
    plateforme::ouvrir_dossier(sorte)
}

/// Le journal, analyse, anonymise, compte et filtre.
///
/// **L'interface ne transmet aucun chemin**, seulement des niveaux, une
/// recherche et le choix d'anonymiser. Cairn lit son propre journal et
/// lui seul, comme pour l'ouverture de dossier du lot 22d : il n'y a rien
/// a valider parce qu'il n'y a rien a composer.
///
/// Le filtrage se fait ici et pas dans l'interface, pour la meme raison
/// que la recherche de serveurs : ce qui merite un test vit dans
/// `cairn-core`. Un niveau inconnu est ignore plutot que refuse, une
/// interface plus recente que le moteur ne doit pas vider la liste.
#[tauri::command]
async fn lire_journal(
    niveaux: Vec<String>,
    recherche: String,
    anonyme: bool,
    limite: usize,
) -> Result<cairn_core::journal::Vue, String> {
    let choisis: Vec<cairn_core::journal::Niveau> = niveaux
        .iter()
        .filter_map(|c| cairn_core::journal::Niveau::depuis_cle(c))
        .collect();
    tauri::async_runtime::spawn_blocking(move || {
        cairn_core::journal::preparer(&journal::lire(), &choisis, &recherche, anonyme, limite)
    })
    .await
    .map_err(|e| {
        journal::alerte(&format!("Lecture du journal interrompue : {e}"));
        "Cairn n'a pas pu lire son journal.".to_string()
    })
}

/// Le rapport a envoyer, **toujours anonymise**.
///
/// Il ne suit pas le choix d'affichage : regarder ses propres chemins sur
/// sa propre machine ne regarde que soi, les coller dans un salon de
/// discussion n'est plus la meme chose, et personne ne relit mille lignes
/// avant de les envoyer.
///
/// Il porte le journal entier et pas la vue filtree. Un rapport ou le
/// declarant a deja ecarte ce qu'il croyait sans rapport est exactement
/// celui ou manque la ligne utile.
#[tauri::command]
async fn rapport_journal() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(rapport)
        .await
        .map_err(|e| {
            journal::alerte(&format!("Preparation du rapport interrompue : {e}"));
            "Cairn n'a pas pu preparer le rapport.".to_string()
        })
}

fn rapport() -> String {
    let entete = vec![
        ("Version", env!("CARGO_PKG_VERSION").to_string()),
        ("Contrat", cairn_core::CONTRAT.to_string()),
        (
            "Systeme",
            format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
        ),
    ];
    cairn_core::journal::exporter(&entete, &journal::lire())
}

/// Ecrit le rapport a cote du journal et rend son nom de fichier.
///
/// Le dossier est celui du journal, choisi par Cairn. Aucune boite de
/// dialogue : elle rendrait un chemin venu de l'exterieur, et c'est
/// precisement ce que le reste du pont s'interdit.
#[tauri::command]
async fn enregistrer_rapport() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let Some(chemin) = journal::chemin() else {
            return Err("Cairn n'a pas de dossier de journal sur ce systeme.".to_string());
        };
        let Some(dossier) = chemin.parent() else {
            return Err("Cairn n'a pas de dossier de journal sur ce systeme.".to_string());
        };
        let nom = format!(
            "rapport-{}.txt",
            chrono::Local::now().format("%Y-%m-%d-%H%M%S")
        );
        std::fs::create_dir_all(dossier).map_err(|e| {
            journal::erreur(&format!("Dossier du rapport : {e}"));
            "Cairn n'a pas pu creer le dossier du rapport.".to_string()
        })?;
        std::fs::write(dossier.join(&nom), rapport()).map_err(|e| {
            journal::erreur(&format!("Ecriture du rapport : {e}"));
            "Cairn n'a pas pu ecrire le rapport.".to_string()
        })?;
        journal::ecrire(&format!("Rapport ecrit : {nom}"));
        Ok(nom)
    })
    .await
    .map_err(|e| {
        journal::alerte(&format!("Ecriture du rapport interrompue : {e}"));
        "Cairn n'a pas pu ecrire le rapport.".to_string()
    })?
}

/// Ouvre le dossier du journal dans l'explorateur.
#[tauri::command]
fn ouvrir_dossier_journal() -> Result<(), String> {
    plateforme::ouvrir_dossier_du_journal()
}

/// Ce que l'ecran Communaute recoit.
#[derive(serde::Serialize)]
struct VueActualites {
    articles: Vec<cairn_core::actualites::ArticleRendu>,
    /// Les comptes par categorie, sur le fil entier et pas sur le filtre.
    comptes: Vec<(cairn_core::actualites::Categorie, usize)>,
    /// Quand le fil affiche a ete assemble. `null` quand il n'y en a pas.
    publie: Option<i64>,
    /// Dit pourquoi le fil affiche n'est pas frais, quand c'est le cas.
    ///
    /// **Une cle et pas une phrase** : l'interface se lit dans deux
    /// langues, Rust n'en parle qu'une. Le detail technique est dans le
    /// journal. Voir `cairn_core::actualites::Souci`.
    ///
    /// Separe de l'absence d'articles : un cache valable avec un reseau
    /// coupe n'est pas une panne, c'est une information. L'ecran montre
    /// les actualites ET la raison, au lieu de choisir entre les deux.
    avertissement: Option<cairn_core::actualites::Souci>,
}

/// Le fil d'actualites, verifie, trie et filtre.
///
/// `forcer` demande d'aller voir le reseau meme si le cache est bon.
/// Sans lui, un cache valable est rendu tout de suite : un launcher qui
/// n'affiche rien parce que le reseau est lent est un launcher qui
/// n'affiche rien la moitie du temps.
#[tauri::command]
async fn actualites(
    forcer: bool,
    langue: String,
    categories: Vec<String>,
    recherche: String,
) -> Result<VueActualites, String> {
    let (fil, avertissement) = actualites::fil(forcer).await;
    let Some(fil) = fil else {
        return Ok(VueActualites {
            articles: Vec::new(),
            comptes: Vec::new(),
            publie: None,
            avertissement,
        });
    };
    let publie = fil.publie;
    // Le decoupage du markdown et le tri sont en Rust, la ou ils sont
    // testes. L'interface ne recoit que des blocs a dessiner.
    let tous = cairn_core::actualites::preparer(&fil, &langue);
    let comptes = cairn_core::actualites::compter(&tous);
    // Une categorie inconnue est ignoree plutot que refusee : une
    // interface plus recente que le moteur ne doit pas vider la liste.
    let choisies: Vec<cairn_core::actualites::Categorie> = categories
        .iter()
        .map(|c| cairn_core::actualites::Categorie::depuis_cle(c))
        .collect();
    let articles = cairn_core::actualites::filtrer(&tous, &choisies, &recherche)
        .into_iter()
        .cloned()
        .collect();
    Ok(VueActualites {
        articles,
        comptes,
        publie: Some(publie),
        avertissement,
    })
}

/// Ouvre un lien d'article dans le navigateur du systeme.
///
/// La validation est refaite en Rust meme si l'analyse du markdown a
/// deja ecarte les liens interdits : ce chemin finit dans un appel
/// systeme qui sait aussi ouvrir un fichier ou un programme.
#[tauri::command]
fn ouvrir_lien(lien: String) -> Result<(), String> {
    actualites::ouvrir_lien(&lien)
}

/// Met ou enleve un serveur des favoris, et rend les reglages a jour.
///
/// Une commande a part plutot qu'un aller-retour par `enregistrer_reglages` :
/// l'etoile doit prendre effet tout de suite, alors que l'ecriture des
/// reglages est volontairement differee. Et la borne sur le nombre de
/// favoris vit dans `cairn-core`, la ou elle est testee, pas dans
/// l'interface.
#[tauri::command]
async fn basculer_favori(
    login: String,
    memoire: tauri::State<'_, Memoire>,
) -> Result<Reglages, String> {
    let mut reglages = memoire.lire();
    cairn_core::reglages::basculer_favori(&mut reglages.favoris, &login);
    memoire.ecrire(reglages.clone());
    let rendu = reglages.clone();
    tauri::async_runtime::spawn_blocking(move || stockage::ecrire(&reglages))
        .await
        .map_err(|e| {
            journal::erreur(&format!("Ecriture des favoris interrompue : {e}"));
            "Cairn n'a pas pu enregistrer ses favoris.".to_string()
        })??;
    Ok(rendu)
}

/// Lance le jeu sur un serveur.
#[tauri::command]
async fn rejoindre(
    login: String,
    titre: String,
    profil: String,
    spectate: bool,
    memoire: tauri::State<'_, Memoire>,
) -> Result<jeu::Rejointe, String> {
    let reglages = memoire.lire();
    tauri::async_runtime::spawn_blocking(move || {
        let systeme = plateforme::Systeme::depuis(&reglages);
        let Some(exe) = systeme.chemin_jeu() else {
            journal::erreur("ManiaPlanet.exe introuvable, lancement abandonne.");
            return Err("ManiaPlanet.exe est introuvable.".to_string());
        };
        // L'etat du jeu est relu ici, et pas repris de l'interface : entre
        // le dernier releve et le clic, le joueur a pu fermer ou ouvrir le
        // jeu, et c'est lui qui decide du chemin de lancement.
        let lance = systeme.jeu_lance();
        jeu::rejoindre(&exe, &profil, &login, &titre, spectate, lance)
    })
    .await
    .map_err(|e| {
        journal::erreur(&format!("Lancement interrompu : {e}"));
        "Le lancement a ete interrompu.".to_string()
    })?
}

/// Ouvre le jeu sur le menu d'un title pack.
///
/// L'action des tuiles de l'ecran Editer. Le jeu ne sait pas s'ouvrir
/// sur son editeur de maps, aucun drapeau ni aucune commande du
/// protocole ne le permet, donc le mieux qu'on puisse faire est de poser
/// le joueur sur le menu du titre qu'il veut editer. L'identifiant de
/// titre vient du nom d'un fichier de `Packs`, et il est verifie par
/// `cairn-core` exactement comme celui d'un lancement sur serveur.
#[tauri::command]
async fn ouvrir_titre(
    titre: String,
    profil: String,
    memoire: tauri::State<'_, Memoire>,
) -> Result<jeu::Rejointe, String> {
    let reglages = memoire.lire();
    tauri::async_runtime::spawn_blocking(move || {
        let systeme = plateforme::Systeme::depuis(&reglages);
        let Some(exe) = systeme.chemin_jeu() else {
            return Err("ManiaPlanet.exe est introuvable.".to_string());
        };
        jeu::ouvrir_titre(&exe, &profil, &titre, systeme.jeu_lance())
    })
    .await
    .map_err(|e| {
        journal::erreur(&format!("Ouverture de titre interrompue : {e}"));
        "L'ouverture a ete interrompue.".to_string()
    })?
}

/// Ouvre le jeu sur ses propres menus, sans rejoindre de serveur.
#[tauri::command]
async fn ouvrir_jeu(
    profil: String,
    memoire: tauri::State<'_, Memoire>,
) -> Result<jeu::Rejointe, String> {
    let reglages = memoire.lire();
    tauri::async_runtime::spawn_blocking(move || {
        let systeme = plateforme::Systeme::depuis(&reglages);
        let Some(exe) = systeme.chemin_jeu() else {
            return Err("ManiaPlanet.exe est introuvable.".to_string());
        };
        jeu::ouvrir(&exe, &profil, systeme.jeu_lance())
    })
    .await
    .map_err(|e| {
        journal::erreur(&format!("Ouverture interrompue : {e}"));
        "L'ouverture a ete interrompue.".to_string()
    })?
}

/// Ce que Cairn a retenu du dernier lancement.
///
/// Au tout premier, la langue du systeme decide de la langue de depart et
/// le reste prend ses valeurs par defaut. Rien ici ne peut faire echouer
/// le demarrage, voir [`stockage::lire`].
#[tauri::command]
fn reglages(memoire: tauri::State<'_, Memoire>) -> Reglages {
    // Le fichier a deja ete lu au demarrage, voir `run`. Ici c'est une
    // copie memoire, donc aucune entree-sortie et aucune raison d'etre
    // asynchrone.
    memoire.lire()
}

/// Enregistre les reglages, et **rend ce qui a ete retenu**.
///
/// Les valeurs sont assainies avant d'etre ecrites, alors qu'elles
/// viennent de notre propre interface : ce qui est ecrit aujourd'hui est
/// relu demain, et c'est a la relecture que le controle compte vraiment.
/// Assainir des deux cotes garantit que le fichier ne contient jamais
/// autre chose que ce que le coeur metier reconnait.
///
/// Le retour n'est pas de la politesse. L'interface propose parfois une
/// valeur que le coeur refuse, un raccourci pose sur une touche
/// reservee par exemple, et elle doit alors montrer ce qui a vraiment
/// ete garde plutot que ce qu'elle esperait. Sans ce retour il faudrait
/// relire les reglages juste apres les avoir ecrits, ou pire,
/// redupliquer la regle de validation cote interface.
#[tauri::command]
async fn enregistrer_reglages(
    valeurs: Reglages,
    memoire: tauri::State<'_, Memoire>,
) -> Result<Reglages, String> {
    let propres = valeurs.assainir();
    // Registre et disque dans le meme saut : les deux sont bloquants.
    let propres = tauri::async_runtime::spawn_blocking(move || {
        let mut propres = propres;
        // Le registre n'est touche que si l'etat demande differe de
        // l'etat reel. Sans ce test, chaque glissement dans le
        // selecteur de couleur y ecrirait une fois.
        if propres.demarrage_auto != demarrage::actif() {
            let _ = demarrage::appliquer(propres.demarrage_auto);
        }
        // Et on repose la verite : si l'ecriture a echoue, l'interrupteur
        // doit montrer l'etat du registre et non celui qu'on esperait.
        propres.demarrage_auto = demarrage::actif();
        stockage::ecrire(&propres).map(|()| propres)
    })
    .await
    .map_err(|e| {
        journal::erreur(&format!("Ecriture des reglages interrompue : {e}"));
        "Cairn n'a pas pu enregistrer ses reglages.".to_string()
    })??;
    memoire.ecrire(propres.clone());
    Ok(propres)
}

/// Ferme l'application.
#[tauri::command]
fn quitter(fenetre: tauri::AppHandle) {
    journal::ecrire("Fermeture demandee par l'interface.");
    fenetre.exit(0);
}

/// Ce que l'interface repond a la question posee par la croix.
///
/// `choix` vaut `reduire` ou `quitter`. `memoriser` vient d'une case
/// **decochee par defaut** : quelqu'un qui repond vite a une question
/// qu'il voit pour la premiere fois ne doit pas s'engager pour toujours
/// sans l'avoir voulu.
///
/// **Fermer Cairn ne touche jamais au jeu.** Il n'y a pas une ligne ici
/// qui s'approche du processus de ManiaPlanet, et c'est volontaire :
/// quelqu'un qui range son launcher pendant une partie ne doit pas
/// perdre sa partie.
#[tauri::command]
async fn fermer_ou_reduire(
    choix: String,
    memoriser: bool,
    application: tauri::AppHandle,
    memoire: tauri::State<'_, Memoire>,
) -> Result<(), String> {
    if memoriser && cairn_core::reglages::FERMETURES.contains(&choix.as_str()) {
        let mut reglages = memoire.lire();
        reglages.fermeture = choix.clone();
        memoire.ecrire(reglages.clone());
        tauri::async_runtime::spawn_blocking(move || stockage::ecrire(&reglages))
            .await
            .map_err(|e| {
                journal::alerte(&format!("Ecriture du choix de fermeture interrompue : {e}"));
                "Cairn n'a pas pu retenir ce choix.".to_string()
            })??;
    }
    if choix == "quitter" {
        journal::ecrire("Fermeture choisie a la croix.");
        application.exit(0);
    } else {
        reduire(&application);
    }
    Ok(())
}

/// Met la fenetre dans la barre des taches.
fn reduire(application: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(fenetre) = application.get_webview_window("main") {
        let _ = fenetre.minimize();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    journal::ecrire(&format!(
        "Cairn {} demarre sur {} / {}.",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));

    tauri::Builder::default()
        // Les reglages sont lus une seule fois, ici, et vivent ensuite en
        // memoire. Cette lecture ne peut pas echouer au sens ou elle
        // rendrait toujours des reglages utilisables, voir stockage::lire.
        .manage(Memoire {
            reglages: Mutex::new(stockage::lire()),
            serveurs: Mutex::new(Vec::new()),
            chemin_jeu: Mutex::new(None),
            empreinte: Mutex::new(None),
            historique: Mutex::new(cairn_core::historique::Historique::depuis(ECRAN_DEPART)),
        })
        // La taille et la position de la fenetre sont retenues d'une session
        // a l'autre. C'est le greffon de l'equipe Tauri, pas une version
        // maison : restaurer une fenetre se casse sur les cas tordus, ecran
        // secondaire debranche depuis, fenetre rendue hors champ, etat
        // maximise. Il ecrit son propre fichier a cote de nos reglages.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // La croix de la fenetre.
        //
        // Trois reponses possibles, et le reglage decide. « demander »
        // est le defaut : imposer l'un des deux autres a quelqu'un qui
        // n'a rien demande serait un piege dans les deux sens, une
        // application qu'on croit fermee et qui tourne encore, ou une
        // session perdue parce qu'on voulait juste la ranger.
        //
        // La question est posee par l'interface et pas par une boite de
        // dialogue du systeme : elle doit porter une case a cocher
        // decochee, et surtout ne pas mettre « Fermer » sous le doigt de
        // quelqu'un qui appuie sur Entree par reflexe.
        .on_window_event(|fenetre, evenement| {
            use tauri::{Emitter, Manager};
            let tauri::WindowEvent::CloseRequested { api, .. } = evenement else {
                return;
            };
            let reglages = fenetre.state::<Memoire>().lire();
            if reglages.fermeture_demande() {
                api.prevent_close();
                let _ = fenetre.emit("fermeture-demandee", ());
            } else if reglages.fermeture_reduit() {
                api.prevent_close();
                let _ = fenetre.minimize();
            }
        })
        .setup(|application| {
            use tauri::Manager;
            // L'etat du demarrage automatique est RELU et non cru sur
            // parole : Windows le montre dans son gestionnaire des
            // taches, ou il se decoche sans nous, et un interrupteur qui
            // ment est pire qu'un interrupteur absent.
            let memoire = application.state::<Memoire>();
            let mut reglages = memoire.lire();
            let reel = demarrage::actif();
            if reglages.demarrage_auto != reel {
                journal::alerte(&format!(
                    "Demarrage automatique : le registre dit {reel}, les reglages disaient {}.",
                    reglages.demarrage_auto
                ));
                reglages.demarrage_auto = reel;
                memoire.ecrire(reglages);
            }
            // Lancee par Windows a l'ouverture de session : on se range
            // dans la barre des taches au lieu de s'ouvrir par dessus ce
            // que la personne etait en train de faire.
            if demarrage::ouvrir_reduit() {
                if let Some(fenetre) = application.get_webview_window("main") {
                    let _ = fenetre.minimize();
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            diagnostic,
            etat,
            veille,
            serveurs,
            vue_serveurs,
            tri_suivant,
            basculer_favori,
            naviguer,
            reculer,
            avancer,
            fermer_ou_reduire,
            glisser_tuile,
            decaler_tuile,
            inventaire,
            ouvrir_dossier,
            actualites,
            ouvrir_lien,
            lire_journal,
            rapport_journal,
            enregistrer_rapport,
            ouvrir_dossier_journal,
            rejoindre,
            ouvrir_jeu,
            ouvrir_titre,
            reglages,
            enregistrer_reglages,
            quitter
        ])
        .run(tauri::generate_context!())
        .expect("Cairn n'a pas pu demarrer");
}
