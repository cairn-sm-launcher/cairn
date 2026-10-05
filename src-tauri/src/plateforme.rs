//! Ce que Cairn demande au systeme.
//!
//! Tout ce qui touche le registre, les processus et le disque est ici, et
//! rien d'autre. Chaque methode se contente d'aller chercher une valeur et
//! de la passer a `cairn-core`, qui decide.
//!
//! Le module compile aussi ailleurs que sous Windows, ou il rend
//! poliment « rien ». Ce n'est pas du support de Linux, c'est ce qui permet
//! de verifier la compilation et les tests de cette caisse sur autre chose
//! qu'un poste Windows.

use cairn_core::{installation, inventaire, profils, Plateforme};

/// L'implementation reelle.
pub struct Systeme {
    /// Le chemin designe a la main dans les reglages, s'il y en a un.
    pub chemin_force: Option<String>,
    /// Le chemin trouve lors d'un releve precedent.
    ///
    /// Un jeu ne se deplace pas entre deux releves. Tant que le fichier
    /// est toujours la, on s'en tient a ce qu'on sait, ce qui evite
    /// d'ouvrir le registre et d'ecrire une ligne de journal a chaque
    /// fois. S'il n'y est plus, la recherche complete reprend ses
    /// droits : un jeu desinstalle doit se voir.
    pub chemin_connu: Option<String>,
}

impl Systeme {
    /// Le systeme tel que les reglages le decrivent.
    pub fn depuis(reglages: &cairn_core::reglages::Reglages) -> Self {
        Self {
            chemin_force: reglages.chemin_jeu.clone(),
            chemin_connu: None,
        }
    }

    /// Le meme, en partant de ce qu'un releve precedent avait trouve.
    pub fn depuis_avec_cache(
        reglages: &cairn_core::reglages::Reglages,
        chemin_connu: Option<String>,
    ) -> Self {
        Self {
            chemin_connu,
            ..Self::depuis(reglages)
        }
    }
}

impl Plateforme for Systeme {
    fn chemin_jeu(&self) -> Option<String> {
        // L'ordre compte. Le chemin designe a la main passe devant tout le
        // reste : s'il est la, c'est que la recherche automatique s'est
        // trompee, et on ne va pas la laisser se tromper a nouveau. Il est
        // quand meme verifie, un jeu deplace ou desinstalle depuis ne doit
        // pas bloquer la detection automatique.
        if let Some(connu) = self
            .chemin_connu
            .clone()
            .filter(|c| std::path::Path::new(c).is_file())
        {
            // Deja trouve, toujours la : rien a chercher, rien a ecrire.
            return Some(connu);
        }
        self.chemin_force
            .clone()
            .filter(|c| std::path::Path::new(c).is_file())
            .or_else(chemin_depuis_registre)
            .or_else(chemin_depuis_repli)
            .inspect(|c| journal_chemin(c))
    }

    fn jeu_lance(&self) -> bool {
        jeu_lance()
    }

    fn profils(&self) -> Vec<profils::Profil> {
        lire_profils()
    }
}

/// Le jeu tourne-t-il, sans passer par un releve complet.
///
/// C'est la seule chose que le coup d'oeil des cinq secondes regarde du
/// cote des processus, voir [`cairn_core::veille`].
pub fn jeu_lance() -> bool {
    processus_present(cairn_core::PROCESSUS_JEU)
}

fn journal_chemin(chemin: &str) {
    crate::journal::ecrire(&format!("Jeu trouve : {chemin}"));
}

// ---------------------------------------------------------------------
//  Le registre
// ---------------------------------------------------------------------

/// Le jeu inscrit lui meme la commande du protocole `maniaplanet://` quand
/// il s'enregistre. C'est la source qui suit l'installation ou qu'elle soit,
/// bibliotheque Steam secondaire comprise.
#[cfg(windows)]
fn chemin_depuis_registre() -> Option<String> {
    use winreg::enums::HKEY_CLASSES_ROOT;
    use winreg::RegKey;

    let racine = RegKey::predef(HKEY_CLASSES_ROOT);
    let cle = racine.open_subkey(r"maniaplanet\shell\open\command").ok()?;
    // La valeur par defaut d'une cle porte le nom vide.
    let commande: String = cle.get_value("").ok()?;
    let chemin = installation::exe_depuis_commande(&commande)?;
    // La valeur peut etre de type REG_EXPAND_SZ et contenir un
    // %ProgramFiles(x86)% non developpe. On ne le developpe pas : ce cas
    // fait echouer le test ci dessous, et on retombe alors sur les
    // emplacements de repli, ce qui est exactement le bon comportement.
    std::path::Path::new(&chemin).is_file().then_some(chemin)
}

#[cfg(not(windows))]
fn chemin_depuis_registre() -> Option<String> {
    None
}

fn chemin_depuis_repli() -> Option<String> {
    let trouve = installation::chemins_de_repli()
        .into_iter()
        .find(|c| std::path::Path::new(c).is_file())
        .map(str::to_string);
    if trouve.is_none() {
        crate::journal::alerte(
            "Jeu introuvable : ni le registre ni les emplacements de repli ne donnent \
             de ManiaPlanet.exe. L'utilisateur devra le designer a la main.",
        );
    }
    trouve
}

// ---------------------------------------------------------------------
//  Le processus
// ---------------------------------------------------------------------

/// Le jeu tourne-t-il.
///
/// La comparaison ignore la casse et l'extension : selon la facon dont le
/// jeu a ete lance, le nom releve est `ManiaPlanet.exe` ou `ManiaPlanet`.
fn processus_present(nom_attendu: &str) -> bool {
    use sysinfo::{ProcessRefreshKind, RefreshKind, System};

    let systeme = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing()),
    );
    systeme.processes().values().any(|processus| {
        let nom = processus.name().to_string_lossy();
        let nom = nom.strip_suffix(".exe").unwrap_or(&nom);
        nom.eq_ignore_ascii_case(nom_attendu)
    })
}

// ---------------------------------------------------------------------
//  Les profils
// ---------------------------------------------------------------------

/// Les dossiers ou le jeu range ses profils, du plus officiel au plus
/// approximatif. Le premier qui contient quelque chose gagne.
fn dossiers_de_profils() -> Vec<std::path::PathBuf> {
    let mut dossiers = Vec::new();
    // Chemin declare par Windows, deja traduit : « Mes Documents » sur une
    // installation francaise. C'est pour ca qu'on ne le compose pas a la
    // main a partir du dossier utilisateur.
    if let Some(documents) = dirs::document_dir() {
        dossiers.push(documents.join("ManiaPlanet").join("Config"));
    }
    if let Some(maison) = dirs::home_dir() {
        let compose = maison.join("Documents").join("ManiaPlanet").join("Config");
        if !dossiers.contains(&compose) {
            dossiers.push(compose);
        }
    }
    dossiers
}

/// De quoi reconnaitre le dossier des profils sans le lire vraiment.
///
/// La date de modification la plus recente et le nombre de fichiers de
/// profil, voir [`cairn_core::veille::Empreinte`]. C'est un `read_dir`
/// sur un dossier qui contient trois fichiers, contre l'enumeration de
/// tous les processus du systeme et une ouverture de registre pour un
/// releve complet.
pub fn empreinte_profils() -> Option<(u64, usize)> {
    for dossier in dossiers_de_profils() {
        let Ok(entrees) = std::fs::read_dir(&dossier) else {
            continue;
        };
        let dates: Vec<u64> = entrees
            .flatten()
            .filter_map(|entree| {
                let nom = entree.file_name().to_string_lossy().into_owned();
                profils::nom_depuis_fichier(&nom)?;
                let modifie = entree.metadata().ok()?.modified().ok()?;
                modifie
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            })
            .collect();
        if dates.is_empty() {
            continue;
        }
        return Some((dates.iter().copied().max().unwrap_or(0), dates.len()));
    }
    None
}

fn lire_profils() -> Vec<profils::Profil> {
    for dossier in dossiers_de_profils() {
        let Ok(entrees) = std::fs::read_dir(&dossier) else {
            continue;
        };

        // On garde la date de modification a cote du nom : elle sert au tri
        // et a la note « Last used ».
        let mut trouves: Vec<(std::time::SystemTime, String)> = entrees
            .flatten()
            .filter_map(|entree| {
                let nom = entree.file_name().to_string_lossy().into_owned();
                profils::nom_depuis_fichier(&nom)?;
                let modifie = entree.metadata().ok()?.modified().ok()?;
                Some((modifie, nom))
            })
            .collect();

        if trouves.is_empty() {
            continue;
        }

        // Du plus recemment utilise au plus ancien : c'est presque toujours
        // le premier que le joueur veut.
        trouves.sort_by_key(|(modifie, _)| std::cmp::Reverse(*modifie));

        let liste: Vec<profils::Profil> = trouves
            .into_iter()
            .filter_map(|(modifie, nom)| {
                let date = chrono::DateTime::<chrono::Local>::from(modifie)
                    .format("%Y-%m-%d")
                    .to_string();
                profils::depuis_fichier(&nom, Some(&date))
            })
            .collect();

        crate::journal::ecrire(&format!(
            "{} profil(s) lu(s) dans {}",
            liste.len(),
            dossier.display()
        ));
        return liste;
    }

    crate::journal::alerte("Aucun profil trouve.");
    Vec::new()
}

// ---------------------------------------------------------------------
//  L'inventaire des fichiers du joueur
// ---------------------------------------------------------------------

/// Ce qu'on refuse de parcourir, quoi qu'il arrive.
///
/// Un dossier de maps peut contenir n'importe quoi, y compris un lien
/// vers ailleurs. Deux bornes donc : la profondeur, et le nombre de
/// fichiers. La seconde n'est pas theorique, un dossier `Downloaded`
/// accumule tout ce qu'on a croise sur un serveur depuis des annees.
const PROFONDEUR_MAX: usize = 4;
const FICHIERS_MAX: usize = 20_000;

/// Les racines possibles du dossier ManiaPlanet de l'utilisateur.
///
/// Meme chaine de repli que pour les profils, et pour la meme raison :
/// sur une installation francaise, Windows declare « Mes Documents », et
/// composer le chemin a la main a partir du dossier utilisateur donnerait
/// « Documents » qui n'existe pas.
fn dossiers_maniaplanet() -> Vec<std::path::PathBuf> {
    let mut dossiers = Vec::new();
    if let Some(documents) = dirs::document_dir() {
        dossiers.push(documents.join("ManiaPlanet"));
    }
    if let Some(maison) = dirs::home_dir() {
        let compose = maison.join("Documents").join("ManiaPlanet");
        if !dossiers.contains(&compose) {
            dossiers.push(compose);
        }
    }
    dossiers
}

/// Le dossier d'une sorte de fichiers, s'il existe.
pub fn dossier_de(sorte: inventaire::Sorte) -> Option<std::path::PathBuf> {
    dossiers_maniaplanet()
        .into_iter()
        .map(|racine| racine.join(sorte.dossier()))
        .find(|chemin| chemin.is_dir())
}

/// Parcourt le dossier d'une sorte et rend son resume.
///
/// Le classement, les comptes et les dates sont calcules par
/// `cairn-core`, ou ils sont testes. Ce qui est ici est le parcours, et
/// lui seul : ouvrir des dossiers, lire des noms et des metadonnees.
///
/// **Aucun fichier n'est ouvert.** Un `.Gbx` n'est jamais lu, on se
/// contente de ce que le systeme donne deja pour lister un dossier.
pub fn inventorier(sorte: inventaire::Sorte) -> inventaire::Inventaire {
    let Some(racine) = dossier_de(sorte) else {
        return inventaire::Inventaire::default();
    };
    let mut entrees = Vec::new();
    parcourir(&racine, "", 0, &mut entrees);
    inventaire::resumer(sorte, &entrees)
}

/// Descend un dossier, en notant le sous-dossier de premier niveau.
///
/// `groupe` est le nom du sous-dossier de PREMIER niveau, et il est donc
/// transmis tel quel aux etages suivants : une map rangee dans
/// `My Maps/01_Elite` compte pour `My Maps`. C'est la granularite de la
/// fiche depliee, qui doit tenir en quelques lignes, pas celle de
/// l'arborescence complete.
fn parcourir(
    dossier: &std::path::Path,
    groupe: &str,
    profondeur: usize,
    entrees: &mut Vec<inventaire::Entree>,
) {
    if profondeur >= PROFONDEUR_MAX || entrees.len() >= FICHIERS_MAX {
        return;
    }
    let Ok(contenu) = std::fs::read_dir(dossier) else {
        return;
    };
    for entree in contenu.flatten() {
        if entrees.len() >= FICHIERS_MAX {
            return;
        }
        let nom = entree.file_name().to_string_lossy().into_owned();
        // `file_type` ne suit pas les liens, contrairement a `metadata`.
        // C'est ce qui empeche un raccourci pose dans le dossier des maps
        // de nous emmener parcourir le disque entier.
        let Ok(sorte_entree) = entree.file_type() else {
            continue;
        };
        if sorte_entree.is_symlink() {
            continue;
        }
        if sorte_entree.is_dir() {
            let sous_groupe = if profondeur == 0 {
                nom.as_str()
            } else {
                groupe
            };
            parcourir(&entree.path(), sous_groupe, profondeur + 1, entrees);
            continue;
        }
        let Ok(infos) = entree.metadata() else {
            continue;
        };
        let modifie = infos
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        entrees.push(inventaire::Entree {
            nom,
            groupe: groupe.to_string(),
            octets: infos.len(),
            modifie,
        });
    }
}

/// Ouvre le dossier d'une sorte dans l'explorateur de fichiers.
///
/// **L'interface n'envoie jamais de chemin**, elle envoie une cle, et
/// c'est cette fonction qui la resout. Il n'y a donc rien a valider,
/// parce qu'il n'y a rien a composer : le seul chemin qui puisse sortir
/// d'ici est l'un des trois que Cairn connait.
pub fn ouvrir_dossier(sorte: inventaire::Sorte) -> Result<(), String> {
    let Some(chemin) = dossier_de(sorte) else {
        return Err("Ce dossier de ManiaPlanet est introuvable.".to_string());
    };
    ouvrir_dans_l_explorateur(&chemin)
}

/// Ouvre le dossier du journal.
///
/// Meme regle que ci-dessus : l'interface ne transmet aucun chemin, elle
/// demande le dossier du journal et Cairn resout le sien.
pub fn ouvrir_dossier_du_journal() -> Result<(), String> {
    let Some(fichier) = crate::journal::chemin() else {
        return Err("Cairn n'a pas de dossier de journal sur ce systeme.".to_string());
    };
    let Some(dossier) = fichier.parent() else {
        return Err("Cairn n'a pas de dossier de journal sur ce systeme.".to_string());
    };
    // Il peut ne pas exister encore si rien n'a jamais ete ecrit, ce qui
    // n'arrive qu'en developpement sans dossier de donnees.
    let _ = std::fs::create_dir_all(dossier);
    ouvrir_dans_l_explorateur(dossier)
}

#[cfg(windows)]
fn ouvrir_dans_l_explorateur(chemin: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(chemin)
        .spawn()
        .map(|_| ())
        .map_err(|e| {
            crate::journal::erreur(&format!("Ouverture du dossier : {e}"));
            "Cairn n'a pas pu ouvrir ce dossier.".to_string()
        })
}

#[cfg(not(windows))]
fn ouvrir_dans_l_explorateur(_chemin: &std::path::Path) -> Result<(), String> {
    Err("L'ouverture d'un dossier n'existe que sous Windows.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_processus_qui_n_existe_pas_n_est_pas_trouve() {
        assert!(!processus_present("CeProcessusNExistePasVraiment"));
    }

    #[test]
    fn les_dossiers_maniaplanet_ne_font_pas_de_doublon() {
        let dossiers = dossiers_maniaplanet();
        let mut uniques = dossiers.clone();
        uniques.sort();
        uniques.dedup();
        assert_eq!(dossiers.len(), uniques.len());
    }

    /// Un dossier de maps bricole pour la duree d'un test.
    fn bac_a_sable(nom: &str) -> std::path::PathBuf {
        let racine = std::env::temp_dir().join(format!("cairn-test-{nom}"));
        let _ = std::fs::remove_dir_all(&racine);
        std::fs::create_dir_all(&racine).unwrap();
        racine
    }

    #[test]
    fn le_parcours_note_le_sous_dossier_de_premier_niveau() {
        // Une map rangee dans « My Maps/01_Elite » compte pour
        // « My Maps » : c'est la granularite de la fiche depliee, pas
        // celle de l'arborescence complete.
        let racine = bac_a_sable("groupes");
        std::fs::create_dir_all(racine.join("My Maps").join("01_Elite")).unwrap();
        std::fs::write(racine.join("RedRock.Map.Gbx"), b"x").unwrap();
        std::fs::write(racine.join("My Maps").join("BXS.Map.Gbx"), b"xx").unwrap();
        std::fs::write(
            racine.join("My Maps").join("01_Elite").join("A.Map.Gbx"),
            b"xxx",
        )
        .unwrap();

        let mut entrees = Vec::new();
        parcourir(&racine, "", 0, &mut entrees);
        entrees.sort_by(|a, b| a.nom.cmp(&b.nom));
        let vu: Vec<(String, String)> = entrees
            .iter()
            .map(|e| (e.nom.clone(), e.groupe.clone()))
            .collect();
        assert_eq!(
            vu,
            [
                ("A.Map.Gbx".to_string(), "My Maps".to_string()),
                ("BXS.Map.Gbx".to_string(), "My Maps".to_string()),
                ("RedRock.Map.Gbx".to_string(), String::new()),
            ]
        );
        let _ = std::fs::remove_dir_all(&racine);
    }

    #[test]
    fn le_parcours_s_arrete_a_la_profondeur_convenue() {
        // Un dossier de maps peut contenir n'importe quoi. La borne de
        // profondeur est ce qui garantit qu'un parcours se termine, quoi
        // qu'on y ait range.
        let racine = bac_a_sable("profondeur");
        let mut chemin = racine.clone();
        for etage in 0..(PROFONDEUR_MAX + 2) {
            chemin = chemin.join(format!("etage{etage}"));
            std::fs::create_dir_all(&chemin).unwrap();
            std::fs::write(chemin.join(format!("m{etage}.Map.Gbx")), b"x").unwrap();
        }

        let mut entrees = Vec::new();
        parcourir(&racine, "", 0, &mut entrees);
        let noms: Vec<&str> = entrees.iter().map(|e| e.nom.as_str()).collect();
        assert!(noms.contains(&"m0.Map.Gbx"));
        // Au dela de la borne, plus rien n'est lu.
        assert!(!noms.contains(&format!("m{}.Map.Gbx", PROFONDEUR_MAX).as_str()));
        let _ = std::fs::remove_dir_all(&racine);
    }

    #[test]
    fn les_dossiers_de_profils_ne_font_pas_de_doublon() {
        let dossiers = dossiers_de_profils();
        let mut uniques = dossiers.clone();
        uniques.sort();
        uniques.dedup();
        assert_eq!(dossiers.len(), uniques.len());
    }
}
