//! Ce que Cairn retient d'une session a l'autre.
//!
//! Le fichier vit dans `%APPDATA%\Cairn\reglages.json`, sa lecture et son
//! ecriture sont dans `cairn-app`. Ce qui est ici, c'est la forme, les
//! valeurs par defaut, et surtout le controle de ce qui remonte du disque.
//!
//! **Pourquoi controler un fichier que l'application a ecrit elle meme.**
//! Parce qu'entre deux lancements il a pu changer. Un joueur l'ouvre pour
//! bricoler, une version precedente y a laisse une valeur qui ne veut plus
//! rien dire, un utilitaire tiers l'ecrase. Et la couleur d'accent ne fait
//! pas que s'afficher : elle part dans un attribut `style`. Une chaine
//! venue du disque qui atterrit dans du CSS sans etre verifiee, c'est la
//! meme famille de probleme que le login de serveur du lot 19b.
//!
//! La regle est donc simple : rien n'est refuse, tout est ramene. Une
//! valeur qu'on ne reconnait pas reprend sa valeur par defaut, et
//! l'application demarre. Un launcher qui refuse de s'ouvrir parce qu'un
//! fichier de preferences est abime serait un mauvais launcher.

use serde::{Deserialize, Serialize};

/// Les trois tuiles de l'ecran Jouer, dans leur ordre d'origine.
pub const TUILES_JOUER: [&str; 3] = ["favorites", "popular", "browse"];

/// Celles de l'ecran Editer, ajoutees au lot 22d.
pub const TUILES_EDITER: [&str; 3] = ["maps", "replays", "packs"];

/// Laquelle des deux grilles de tuiles on range.
///
/// Les deux ecrans partagent le meme geste et les memes regles, ils ne
/// different que par la liste des tuiles qui existent. Passer la grille
/// plutot que la liste elle meme garde l'interface hors du coup : elle
/// envoie « jouer » ou « editer », jamais un ensemble de tuiles qu'elle
/// aurait compose de son cote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grille {
    Jouer,
    Editer,
}

impl Grille {
    /// Les tuiles qui existent dans cette grille, dans leur ordre d'origine.
    pub fn tuiles(self) -> &'static [&'static str] {
        match self {
            Grille::Jouer => &TUILES_JOUER,
            Grille::Editer => &TUILES_EDITER,
        }
    }

    /// La grille designee par une cle, ou rien si elle n'en designe aucune.
    pub fn depuis_cle(cle: &str) -> Option<Self> {
        match cle {
            "jouer" => Some(Grille::Jouer),
            "editer" => Some(Grille::Editer),
            _ => None,
        }
    }
}

/// La couleur d'accent par defaut, le teal Storm.
pub const ACCENT_DEFAUT: &str = "#20E3B2";

const LANGUES: [&str; 2] = ["fr", "en"];
const PALETTES: [&str; 5] = ["storm", "aurora", "ember", "slate", "custom"];
const INTENSITES: [&str; 3] = ["soft", "medium", "vivid"];
const VITESSES: [&str; 3] = ["slow", "normal", "fast"];

/// Les trois densites d'affichage, demandees par Flow au lot 19d.
///
/// `compact` est l'espacement d'origine, celui des lots 17 a 19c.
/// `equilibre` est celui qu'il a decrit : de l'air entre le bandeau, les
/// tuiles et les actualites. `aere` en met davantage. C'est `equilibre`
/// qui s'applique au premier lancement.
pub const DENSITES: [&str; 3] = ["compact", "equilibre", "aere"];

/// Les trois etats du filtre mot de passe.
///
/// `tous` ne filtre rien, `libres` ne garde que ce qu'on peut rejoindre
/// sans rien demander a personne, `prives` ne garde que le contraire.
pub const MOTS_DE_PASSE: [&str; 3] = ["tous", "libres", "prives"];

/// Ce que fait la croix de la fenetre.
///
/// `demander` pose la question une fois par fermeture, `reduire` met
/// dans la barre des taches, `quitter` ferme. C'est `demander` au
/// depart : imposer l'un des deux autres a quelqu'un qui n'a rien
/// demande serait un piege dans les deux sens, une application qu'on
/// croit fermee et qui tourne encore, ou une session perdue parce qu'on
/// voulait juste la ranger.
pub const FERMETURES: [&str; 3] = ["demander", "reduire", "quitter"];

/// Le nombre de favoris qu'on accepte de relire depuis le disque.
///
/// Personne n'a deux cents serveurs favoris. La borne n'est pas la pour
/// brider l'usage, elle est la pour qu'un fichier trafique ne fasse pas
/// tourner le filtre sur une liste sans fin.
pub const FAVORIS_MAX: usize = 200;

/// Les trois teintes du fond personnalise, quand la palette est `custom`.
pub const FOND_DEFAUT: [&str; 3] = ["#20E3B2", "#5B8CFF", "#FF6B4A"];

/// Le nombre d'emplacements de themes.
///
/// Trois, comme dans la maquette. Assez pour garder un theme clair, un
/// sombre et un d'essai, assez peu pour qu'on se souvienne de ce qu'il y
/// a dedans sans les nommer tous.
pub const EMPLACEMENTS_THEMES: usize = 3;

/// L'apparence, c'est a dire ce qu'un emplacement de theme retient.
///
/// Elle existe comme structure separee parce qu'elle est enregistree et
/// rechargee en bloc. Les champs restent aussi a plat dans [`Reglages`],
/// et c'est volontaire : les deplacer dans un sous-objet changerait la
/// forme du fichier, et le premier lancement apres la mise a jour
/// rendrait a chacun sa couleur par defaut sans explication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Apparence {
    pub accent: String,
    pub fond_palette: String,
    pub fond_intensite: String,
    pub fond_vitesse: String,
    pub fond_personnalise: [String; 3],
}

impl Default for Apparence {
    fn default() -> Self {
        Self {
            accent: ACCENT_DEFAUT.to_string(),
            fond_palette: "storm".to_string(),
            fond_intensite: "medium".to_string(),
            fond_vitesse: "slow".to_string(),
            fond_personnalise: FOND_DEFAUT.map(str::to_string),
        }
    }
}

impl Apparence {
    pub fn assainir(mut self) -> Self {
        let defaut = Self::default();
        if !couleur_valide(&self.accent) {
            self.accent = defaut.accent;
        }
        if !PALETTES.contains(&self.fond_palette.as_str()) {
            self.fond_palette = defaut.fond_palette;
        }
        if !INTENSITES.contains(&self.fond_intensite.as_str()) {
            self.fond_intensite = defaut.fond_intensite;
        }
        if !VITESSES.contains(&self.fond_vitesse.as_str()) {
            self.fond_vitesse = defaut.fond_vitesse;
        }
        for (teinte, secours) in self.fond_personnalise.iter_mut().zip(FOND_DEFAUT) {
            if !couleur_valide(teinte) {
                *teinte = secours.to_string();
            }
        }
        self
    }
}

/// Un emplacement de theme.
///
/// Vide tant qu'on n'y a rien enregistre. Le nom est libre : un
/// emplacement sans nom s'affiche avec son numero, ce qui vaut mieux
/// que d'imposer un nom avant de pouvoir enregistrer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub nom: String,
    /// `None` quand l'emplacement est vide.
    pub apparence: Option<Apparence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reglages {
    /// `fr` ou `en`. Au tout premier lancement c'est la langue du systeme
    /// qui decide, et `cairn-app` la passe a [`Reglages::premier_lancement`].
    pub langue: String,
    /// Couleur d'accent, en `#RRGGBB`.
    pub accent: String,
    pub fond_palette: String,
    pub fond_intensite: String,
    pub fond_vitesse: String,
    /// Les trois teintes utilisees quand `fond_palette` vaut `custom`.
    pub fond_personnalise: [String; 3],
    /// L'interrupteur general des animations.
    pub animations: bool,
    /// Ce que fait la croix de la fenetre, une des [`FERMETURES`].
    pub fermeture: String,
    /// Cairn s'ouvre avec la session Windows.
    ///
    /// La verite est dans le registre et pas ici : ce champ n'est qu'un
    /// reflet, remis en accord au demarrage. Quelqu'un qui decoche
    /// Cairn dans l'onglet Demarrage du gestionnaire des taches doit
    /// voir l'interrupteur suivre, sinon elle ment.
    pub demarrage_auto: bool,
    /// Rendre les couleurs et les styles que le jeu met dans les noms.
    ///
    /// Vrai par defaut : un serveur se reconnait a son nom colore, c'est
    /// son identite. Mais certains noms sont des sapins de Noel, et
    /// quelqu'un qui parcourt la liste tous les soirs peut preferer une
    /// colonne d'une seule couleur, qui se balaye plus vite. Le reglage
    /// ne change que l'affichage : le nom reste le meme, la recherche et
    /// le tri portent de toute facon sur le nom nu.
    pub couleurs_jeu: bool,
    /// L'espacement general, une des valeurs de [`DENSITES`].
    ///
    /// Volontairement hors de [`Apparence`], donc hors des emplacements
    /// de themes : un theme dit une couleur et une ambiance, pas une
    /// densite. Charger un theme pour sa couleur et voir toute la page se
    /// resserrer serait une surprise, et une mauvaise.
    pub densite: String,
    /// L'ordre des tuiles de l'ecran Jouer.
    ///
    /// Le champ garde son nom sans suffixe alors que sa voisine en porte
    /// un. C'est laid et c'est voulu : le renommer en
    /// `ordre_tuiles_jouer` changerait la forme du fichier de reglages,
    /// et le premier lancement apres la mise a jour rendrait a chacun
    /// l'ordre par defaut sans explication. Meme raisonnement que pour
    /// les champs d'apparence restes a plat au lot 19c-3.
    pub ordre_tuiles: Vec<String>,
    /// L'ordre des tuiles de l'ecran Editer, depuis le lot 22d.
    #[serde(default)]
    pub ordre_tuiles_editer: Vec<String>,
    /// Le profil retenu au dernier lancement. Il est recoupe avec les
    /// profils qui existent vraiment, voir [`crate::profils::profil_actif`].
    pub profil: Option<String>,
    /// Le chemin de `ManiaPlanet.exe` designe a la main.
    ///
    /// Vide dans l'immense majorite des cas : le registre repond, et c'est
    /// la source qui suit l'installation ou qu'elle soit. Ce champ est
    /// pour les autres, ceux dont la cle de protocole a ete ecrasee par un
    /// autre logiciel ou n'a jamais ete ecrite. Sans lui, un joueur dont
    /// le jeu n'est pas trouve n'a aucun recours.
    pub chemin_jeu: Option<String>,
    /// Passe a `true` quand l'assistant de premier lancement a ete mene a
    /// son terme. C'est lui qui decide si Cairn s'ouvre sur l'assistant.
    pub assistant_fait: bool,

    // --- L'ecran Parcourir les serveurs -----------------------------
    //
    // L'etat des filtres est retenu d'une session a l'autre : quelqu'un
    // qui ne joue qu'a Obstacle ne doit pas avoir a le redire chaque
    // soir. La recherche, elle, n'est pas retenue : c'est un geste du
    // moment, et la retrouver au lancement suivant donnerait une liste
    // amputee sans qu'on comprenne pourquoi.
    /// La touche qui ramene vers l'onglet de gauche, par sa **position**
    /// et non par sa lettre. Voir [`crate::touches`].
    pub touche_precedent: String,
    /// La touche qui pousse vers l'onglet de droite.
    pub touche_suivant: String,
    /// Les boutons lateraux de la souris changent aussi d'onglet.
    pub souris_laterale: bool,
    /// Les emplacements de themes de l'onglet Apparence.
    pub themes: Vec<Theme>,

    /// Le mode retenu dans la barre de filtres. Vide veut dire tous.
    pub serveurs_mode: String,
    /// Sur quoi la liste est triee.
    pub serveurs_tri: crate::serveurs::Tri,
    /// Le groupe des serveurs ou il y a du monde est-il deplie.
    pub serveurs_peuples_ouverts: bool,
    /// Le groupe des serveurs vides est-il deplie. Ferme par defaut :
    /// c'est la longue trainee, pas ce qu'on vient chercher.
    pub serveurs_vides_ouverts: bool,
    /// Le sens du tri. Chaque critere a le sien au premier clic, voir
    /// [`crate::serveurs::ordre_naturel_croissant`].
    pub serveurs_tri_croissant: bool,
    /// Le filtre mot de passe, une des valeurs de [`MOTS_DE_PASSE`].
    pub serveurs_mot_de_passe: String,
    /// Ne montrer que les favoris.
    pub serveurs_favoris: bool,
    /// Les serveurs mis en favori, par leur **login**.
    ///
    /// Le login et pas le nom : un serveur se renomme, change de mode,
    /// disparait une semaine et revient. Le login, lui, ne bouge pas.
    /// Un favori hors ligne n'est donc pas perdu, il est juste absent de
    /// la liste du soir.
    pub favoris: Vec<String>,
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            langue: "en".to_string(),
            accent: ACCENT_DEFAUT.to_string(),
            fond_palette: "storm".to_string(),
            fond_intensite: "medium".to_string(),
            fond_vitesse: "slow".to_string(),
            fond_personnalise: FOND_DEFAUT.map(str::to_string),
            animations: true,
            fermeture: "demander".to_string(),
            demarrage_auto: false,
            couleurs_jeu: true,
            densite: "equilibre".to_string(),
            ordre_tuiles: TUILES_JOUER.iter().map(|t| t.to_string()).collect(),
            ordre_tuiles_editer: TUILES_EDITER.iter().map(|t| t.to_string()).collect(),
            profil: None,
            chemin_jeu: None,
            touche_precedent: crate::touches::PRECEDENT_DEFAUT.to_string(),
            touche_suivant: crate::touches::SUIVANT_DEFAUT.to_string(),
            souris_laterale: true,
            themes: vec![Theme::default(); EMPLACEMENTS_THEMES],
            serveurs_mode: String::new(),
            serveurs_tri: crate::serveurs::Tri::default(),
            serveurs_peuples_ouverts: true,
            serveurs_vides_ouverts: false,
            serveurs_tri_croissant: crate::serveurs::ordre_naturel_croissant(
                crate::serveurs::Tri::Joueurs,
            ),
            serveurs_mot_de_passe: "tous".to_string(),
            serveurs_favoris: false,
            favoris: Vec::new(),
            assistant_fait: false,
        }
    }
}

impl Reglages {
    /// L'apparence courante, extraite des champs a plat.
    pub fn apparence(&self) -> Apparence {
        Apparence {
            accent: self.accent.clone(),
            fond_palette: self.fond_palette.clone(),
            fond_intensite: self.fond_intensite.clone(),
            fond_vitesse: self.fond_vitesse.clone(),
            fond_personnalise: self.fond_personnalise.clone(),
        }
    }

    /// Repose une apparence sur les champs a plat. C'est ce que fait
    /// « Charger » sur un emplacement de theme.
    pub fn appliquer_apparence(&mut self, apparence: &Apparence) {
        self.accent = apparence.accent.clone();
        self.fond_palette = apparence.fond_palette.clone();
        self.fond_intensite = apparence.fond_intensite.clone();
        self.fond_vitesse = apparence.fond_vitesse.clone();
        self.fond_personnalise = apparence.fond_personnalise.clone();
    }

    /// Les reglages d'un poste ou Cairn n'a jamais tourne.
    ///
    /// `langue_systeme` est la locale de l'OS, sous n'importe quelle forme :
    /// `fr`, `fr-FR`, `fr_FR.UTF-8`. Seule la premiere etiquette compte, et
    /// elle doit valoir `fr` exactement : un `starts_with("fr")` prendrait
    /// « french » ou « frisian » pour du francais. L'utilisateur garde la
    /// main ensuite, dans les reglages.
    pub fn premier_lancement(langue_systeme: &str) -> Self {
        let fr = langue_systeme
            .split(['-', '_', '.', '@'])
            .next()
            .is_some_and(|etiquette| etiquette.eq_ignore_ascii_case("fr"));
        Self {
            langue: if fr { "fr" } else { "en" }.to_string(),
            ..Self::default()
        }
    }

    /// Ramene chaque champ dans son domaine.
    ///
    /// Ne rend jamais d'erreur : une valeur qu'on ne reconnait pas est
    /// remplacee par sa valeur par defaut, en silence. Le seul cas qui
    /// merite une trace est l'ordre des tuiles, voir plus bas.
    pub fn assainir(mut self) -> Self {
        let defaut = Self::default();

        if !LANGUES.contains(&self.langue.as_str()) {
            self.langue = defaut.langue;
        }
        // L'apparence est assainie par sa propre structure, pour que les
        // memes regles s'appliquent aux reglages courants et a ce qui
        // dort dans un emplacement de theme.
        let apparence = self.apparence().assainir();
        self.appliquer_apparence(&apparence);
        self.ordre_tuiles = ordre_assaini(Grille::Jouer, &self.ordre_tuiles);
        self.ordre_tuiles_editer = ordre_assaini(Grille::Editer, &self.ordre_tuiles_editer);
        // Le mode n'est pas verifie contre une liste : les modes
        // disponibles changent avec ce qui est en ligne ce soir la, et
        // un mode absent ce soir ne doit pas etre efface du reglage. La
        // barre de filtres retombe simplement sur « tous » quand elle
        // ne retrouve pas le mode retenu.
        self.serveurs_mode = self.serveurs_mode.trim().to_string();
        if !MOTS_DE_PASSE.contains(&self.serveurs_mot_de_passe.as_str()) {
            self.serveurs_mot_de_passe = defaut.serveurs_mot_de_passe;
        }
        if !DENSITES.contains(&self.densite.as_str()) {
            self.densite = defaut.densite;
        }
        if !FERMETURES.contains(&self.fermeture.as_str()) {
            self.fermeture = defaut.fermeture;
        }
        self.favoris = favoris_assainis(&self.favoris);

        let (precedent, suivant) =
            crate::touches::assainir(&self.touche_precedent, &self.touche_suivant);
        self.touche_precedent = precedent;
        self.touche_suivant = suivant;

        // Toujours exactement le nombre d'emplacements prevu : un fichier
        // qui en porterait moins laisserait une rangee manquante, un qui
        // en porterait plus ferait apparaitre des emplacements fantomes.
        self.themes.truncate(EMPLACEMENTS_THEMES);
        self.themes.resize(EMPLACEMENTS_THEMES, Theme::default());
        for theme in &mut self.themes {
            theme.nom = theme.nom.trim().chars().take(40).collect();
            theme.apparence = theme.apparence.take().map(Apparence::assainir);
        }
        if self.profil.as_deref().is_some_and(str::is_empty) {
            self.profil = None;
        }
        // Un chemin vide ou fait d'espaces vaut « pas de chemin » : sinon
        // il passerait devant le registre et ferait echouer la recherche.
        if self
            .chemin_jeu
            .as_deref()
            .is_some_and(|c| c.trim().is_empty())
        {
            self.chemin_jeu = None;
        }
        self
    }
}

/// Une couleur acceptable : exactement `#` suivi de six chiffres
/// hexadecimaux.
///
/// Volontairement etroit. Les formes courtes, les noms CSS et les
/// fonctions `rgb()` sont toutes refusees : aucune n'est produite par
/// l'interface, donc en voir une signifie que le fichier a ete modifie
/// ailleurs. Et c'est cette chaine qui part dans un attribut `style`.
pub fn couleur_valide(valeur: &str) -> bool {
    let Some(chiffres) = valeur.strip_prefix('#') else {
        return false;
    };
    chiffres.len() == 6 && chiffres.bytes().all(|c| c.is_ascii_hexdigit())
}

/// Ce qu'un filtre mot de passe demande, sous la forme que
/// [`crate::serveurs::Filtre`] attend.
///
/// Une valeur inconnue ne filtre rien plutot que de filtrer au hasard :
/// elle a normalement ete ramenee par [`Reglages::assainir`], et si elle
/// ne l'a pas ete, cacher la moitie de la liste sans raison visible
/// serait le pire des deux.
pub fn prive_demande(mot_de_passe: &str) -> Option<bool> {
    match mot_de_passe {
        "libres" => Some(false),
        "prives" => Some(true),
        _ => None,
    }
}

impl Reglages {
    /// Vrai quand la croix doit poser la question.
    pub fn fermeture_demande(&self) -> bool {
        self.fermeture == "demander"
    }

    /// Vrai quand la croix doit reduire plutot que fermer.
    pub fn fermeture_reduit(&self) -> bool {
        self.fermeture == "reduire"
    }

    /// Ce que le filtre mot de passe retenu demande.
    pub fn prive_demande(&self) -> Option<bool> {
        prive_demande(&self.serveurs_mot_de_passe)
    }
}

/// Remet la liste des favoris d'aplomb.
///
/// Elle vient du disque comme le reste. On enleve les blancs, on jette
/// les lignes vides et les doublons, on borne la longueur d'un login et
/// le nombre de favoris. L'ordre d'ajout est conserve : c'est celui que
/// l'utilisateur connait.
fn favoris_assainis(demande: &[String]) -> Vec<String> {
    let mut favoris: Vec<String> = Vec::new();
    for login in demande {
        let login: String = login.trim().chars().take(60).collect();
        if login.is_empty() || favoris.contains(&login) {
            continue;
        }
        favoris.push(login);
        if favoris.len() == FAVORIS_MAX {
            break;
        }
    }
    favoris
}

/// Ajoute ou retire un favori, et dit ce qu'il en est apres coup.
///
/// Un seul chemin pour les deux sens : l'etoile de la liste est un
/// interrupteur, pas deux boutons. Rend `true` quand le serveur est
/// desormais en favori.
pub fn basculer_favori(favoris: &mut Vec<String>, login: &str) -> bool {
    let login = login.trim();
    if login.is_empty() {
        return false;
    }
    if let Some(i) = favoris.iter().position(|f| f == login) {
        favoris.remove(i);
        return false;
    }
    if favoris.len() >= FAVORIS_MAX {
        return false;
    }
    favoris.push(login.to_string());
    true
}

/// Deplace une tuile la ou une autre se trouve.
///
/// `avant` dit de quel cote de la cible on lache : c'est la moitie de
/// la tuile survolee qui le decide, et c'est ce qui rend un
/// glisser-deposer previsible. Sans ce cote, lacher a gauche ou a
/// droite d'une tuile donnerait le meme resultat, et on ne saurait
/// jamais ou la tuile va tomber avant de l'avoir lachee.
///
/// **Le retrait avant l'insertion.** On enleve d'abord la tuile, puis on
/// insere, ce qui decale d'un cran toutes les places situees apres
/// l'ancienne. Oublier ce decalage donne un resultat juste dans un sens
/// et faux dans l'autre, ce qui est le genre de defaut qu'on met une
/// soiree a voir parce qu'un essai sur deux marche.
pub fn deplacer_tuile(
    grille: Grille,
    ordre: &[String],
    depuis: &str,
    vers: &str,
    avant: bool,
) -> Vec<String> {
    let mut sortie = ordre.to_vec();
    if depuis == vers {
        return ordre_assaini(grille, &sortie);
    }
    let (Some(i), Some(j)) = (
        sortie.iter().position(|t| t == depuis),
        sortie.iter().position(|t| t == vers),
    ) else {
        return ordre_assaini(grille, &sortie);
    };
    let tuile = sortie.remove(i);
    let mut cible = if j > i { j - 1 } else { j };
    if !avant {
        cible += 1;
    }
    sortie.insert(cible.min(sortie.len()), tuile);
    ordre_assaini(grille, &sortie)
}

/// Une tuile portee au dessus d'une autre : elle prend sa place.
///
/// C'est la regle du glisser au pointeur, et elle n'a pas de cote. Tant
/// que le bouton est enfonce, la tuile portee occupe la case de celle
/// qu'elle survole et l'autre glisse d'un cran pour lui faire de la
/// place. Il n'y a donc rien a deviner : la grille montre le resultat
/// pendant le geste, et le lacher ne fait que le garder.
///
/// L'ancien glisser demandait de quel cote de la cible on lachait, parce
/// qu'un trait dessine entre deux tuiles est la seule chose qu'on puisse
/// montrer quand la grille, elle, ne bouge pas. Une grille qui se
/// reorganise en direct dit la meme chose en mieux, et sans vocabulaire.
///
/// Le `avant` passe a [`deplacer_tuile`] se deduit donc du sens du
/// deplacement : vers la gauche on se pose devant la cible, vers la
/// droite on se pose derriere elle. Dans les deux cas la tuile portee
/// atterrit tres exactement sur l'ancienne case de la cible.
pub fn glisser_sur(grille: Grille, ordre: &[String], depuis: &str, vers: &str) -> Vec<String> {
    let (Some(i), Some(j)) = (
        ordre.iter().position(|t| t == depuis),
        ordre.iter().position(|t| t == vers),
    ) else {
        return ordre_assaini(grille, ordre);
    };
    deplacer_tuile(grille, ordre, depuis, vers, j < i)
}

/// Decale une tuile d'un cran, pour les fleches et le clavier.
///
/// Un cran, donc un echange avec la voisine, ce qui revient au meme
/// qu'un deplacement quand les deux sont cote a cote. Au bord, rien ne
/// bouge : une liste qui boucle ferait revenir la premiere tuile en
/// dernier sans qu'on l'ait demande.
pub fn decaler_tuile(grille: Grille, ordre: &[String], id: &str, pas: i32) -> Vec<String> {
    let mut sortie = ordre.to_vec();
    let Some(i) = sortie.iter().position(|t| t == id) else {
        return ordre_assaini(grille, &sortie);
    };
    let j = i as i32 + pas;
    if j >= 0 && (j as usize) < sortie.len() {
        sortie.swap(i, j as usize);
    }
    ordre_assaini(grille, &sortie)
}

/// Remet l'ordre des tuiles d'aplomb.
///
/// On garde les tuiles connues dans l'ordre demande, on jette ce qu'on ne
/// connait pas, et on rajoute a la fin celles qui manquent. Une tuile
/// disparue de la liste, par exemple parce qu'une version future en a
/// retire une puis qu'on est revenu en arriere, ne doit pas faire
/// disparaitre la tuile de l'ecran.
fn ordre_assaini(grille: Grille, demande: &[String]) -> Vec<String> {
    let tuiles = grille.tuiles();
    let mut ordre: Vec<String> = Vec::with_capacity(tuiles.len());
    for tuile in demande {
        if tuiles.contains(&tuile.as_str()) && !ordre.contains(tuile) {
            ordre.push(tuile.clone());
        }
    }
    for tuile in tuiles {
        if !ordre.iter().any(|t| t == tuile) {
            ordre.push(tuile.to_string());
        }
    }
    ordre
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mots(liste: &[&str]) -> Vec<String> {
        liste.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn les_valeurs_par_defaut_se_survivent_a_elles_memes() {
        let d = Reglages::default();
        assert_eq!(d.clone().assainir(), d);
    }

    #[test]
    fn la_langue_du_systeme_decide_au_premier_lancement() {
        for locale in ["fr", "fr-FR", "FR_fr.UTF-8", "fr_CA"] {
            assert_eq!(Reglages::premier_lancement(locale).langue, "fr", "{locale}");
        }
        for locale in ["en", "en-GB", "de-DE", "", "french"] {
            assert_eq!(Reglages::premier_lancement(locale).langue, "en", "{locale}");
        }
    }

    #[test]
    fn une_couleur_doit_etre_six_chiffres_hexadecimaux() {
        assert!(couleur_valide("#20E3B2"));
        assert!(couleur_valide("#000000"));
        assert!(couleur_valide("#abcdef"));

        assert!(!couleur_valide("#fff"));
        assert!(!couleur_valide("20E3B2"));
        assert!(!couleur_valide("#20E3B"));
        assert!(!couleur_valide("#20E3B22"));
        assert!(!couleur_valide("red"));
        assert!(!couleur_valide("rgb(1,2,3)"));
        assert!(!couleur_valide(""));
    }

    #[test]
    fn une_couleur_qui_sortirait_de_son_attribut_est_ecartee() {
        // Le cas qui justifie tout ce module : une chaine venue du disque
        // qui fermerait l'attribut style pour en ouvrir un autre.
        let abime = Reglages {
            accent: "#000;background:url(http://ailleurs)".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(abime.accent, ACCENT_DEFAUT);
    }

    #[test]
    fn les_valeurs_inconnues_reprennent_leur_defaut() {
        let abime = Reglages {
            langue: "klingon".to_string(),
            fond_palette: "neon".to_string(),
            fond_intensite: "enorme".to_string(),
            fond_vitesse: "tres vite".to_string(),
            ..Reglages::default()
        }
        .assainir();
        let d = Reglages::default();
        assert_eq!(abime.langue, d.langue);
        assert_eq!(abime.fond_palette, d.fond_palette);
        assert_eq!(abime.fond_intensite, d.fond_intensite);
        assert_eq!(abime.fond_vitesse, d.fond_vitesse);
    }

    #[test]
    fn seule_la_teinte_fautive_est_remplacee() {
        let r = Reglages {
            fond_personnalise: [
                "#111111".to_string(),
                "pas une couleur".to_string(),
                "#333333".to_string(),
            ],
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.fond_personnalise[0], "#111111");
        assert_eq!(r.fond_personnalise[1], FOND_DEFAUT[1]);
        assert_eq!(r.fond_personnalise[2], "#333333");
    }

    #[test]
    fn l_ordre_des_tuiles_garde_ce_qu_il_peut_et_complete_le_reste() {
        // un ordre legitime passe intact
        let r = Reglages {
            ordre_tuiles: mots(&["browse", "favorites", "popular"]),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.ordre_tuiles, mots(&["browse", "favorites", "popular"]));

        // une inconnue est jetee, une manquante revient a la fin
        let r = Reglages {
            ordre_tuiles: mots(&["browse", "inconnue"]),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.ordre_tuiles, mots(&["browse", "favorites", "popular"]));

        // un doublon ne duplique pas la tuile
        let r = Reglages {
            ordre_tuiles: mots(&["popular", "popular", "popular"]),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.ordre_tuiles, mots(&["popular", "favorites", "browse"]));

        // une liste vide rend l'ordre d'origine
        let r = Reglages {
            ordre_tuiles: Vec::new(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.ordre_tuiles, mots(&TUILES_JOUER));
    }

    #[test]
    fn un_profil_vide_vaut_pas_de_profil() {
        let r = Reglages {
            profil: Some(String::new()),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.profil, None);
    }

    #[test]
    fn un_chemin_de_jeu_vide_vaut_pas_de_chemin() {
        for vide in ["", "   ", "\t"] {
            let r = Reglages {
                chemin_jeu: Some(vide.to_string()),
                ..Reglages::default()
            }
            .assainir();
            assert_eq!(r.chemin_jeu, None, "{vide:?}");
        }
        // Un chemin renseigne passe intact, la verification qu'il existe
        // vraiment appartient a celui qui touche le disque.
        let r = Reglages {
            chemin_jeu: Some(r"D:\Jeux\ManiaPlanet\ManiaPlanet.exe".to_string()),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(
            r.chemin_jeu.as_deref(),
            Some(r"D:\Jeux\ManiaPlanet\ManiaPlanet.exe")
        );
    }

    #[test]
    fn un_fichier_ampute_se_complete_tout_seul() {
        // serde(default) comble les champs absents : une version future qui
        // ajoute un reglage lira sans broncher un fichier ecrit avant elle.
        // Delimiteur a deux dieses : la valeur contient elle meme `"#`, qui
        // fermerait une chaine brute simple.
        let r: Reglages = serde_json::from_str(r##"{"accent":"#FF6B4A"}"##).unwrap();
        assert_eq!(r.accent, "#FF6B4A");
        assert_eq!(r.fond_palette, "storm");
        assert!(!r.assistant_fait);
    }

    #[test]
    fn un_aller_retour_par_le_json_ne_perd_rien() {
        let avant = Reglages {
            langue: "fr".to_string(),
            accent: "#A06BFF".to_string(),
            profil: Some("Xirdrak".to_string()),
            assistant_fait: true,
            animations: false,
            ordre_tuiles: mots(&["popular", "browse", "favorites"]),
            ..Reglages::default()
        };
        let texte = serde_json::to_string(&avant).unwrap();
        let apres: Reglages = serde_json::from_str(&texte).unwrap();
        assert_eq!(avant, apres.assainir());
    }

    #[test]
    fn les_filtres_de_la_liste_des_serveurs_sont_retenus() {
        let r = Reglages {
            serveurs_mode: "  Obstacle  ".to_string(),
            serveurs_tri: crate::serveurs::Tri::Nom,
            serveurs_vides_ouverts: true,
            serveurs_peuples_ouverts: false,
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.serveurs_mode, "Obstacle");
        assert_eq!(r.serveurs_tri, crate::serveurs::Tri::Nom);
        assert!(r.serveurs_vides_ouverts);
        assert!(!r.serveurs_peuples_ouverts);
    }

    #[test]
    fn un_mode_absent_ce_soir_la_reste_dans_les_reglages() {
        // Les modes disponibles changent avec ce qui est en ligne. Un
        // mode qu'on n'a pas vu depuis une semaine ne doit pas etre
        // efface du reglage pour autant : c'est la barre de filtres qui
        // retombe sur « tous », pas le fichier qui oublie.
        let r = Reglages {
            serveurs_mode: "Joust".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.serveurs_mode, "Joust");
    }

    #[test]
    fn un_fichier_ecrit_avant_l_ecran_des_serveurs_prend_les_defauts() {
        let r: Reglages = serde_json::from_str(r##"{"accent":"#20E3B2"}"##).unwrap();
        assert_eq!(r.serveurs_mode, "");
        assert_eq!(r.serveurs_tri, crate::serveurs::Tri::Joueurs);
        assert!(r.serveurs_peuples_ouverts);
        assert!(!r.serveurs_vides_ouverts);
    }

    #[test]
    fn les_touches_par_defaut_sont_des_positions() {
        let d = Reglages::default();
        assert_eq!(d.touche_precedent, crate::touches::PRECEDENT_DEFAUT);
        assert_eq!(d.touche_suivant, crate::touches::SUIVANT_DEFAUT);
        assert!(d.souris_laterale);
    }

    #[test]
    fn un_raccourci_abime_reprend_son_defaut() {
        let r = Reglages {
            touche_precedent: "Tab".to_string(),
            touche_suivant: "KeyD".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.touche_precedent, crate::touches::PRECEDENT_DEFAUT);
        assert_eq!(r.touche_suivant, "KeyD");
    }

    #[test]
    fn il_y_a_toujours_exactement_trois_emplacements() {
        // Un fichier qui en porte moins laisserait une rangee manquante,
        // un qui en porte plus ferait des emplacements fantomes.
        for combien in [0, 1, 3, 9] {
            let r = Reglages {
                themes: vec![Theme::default(); combien],
                ..Reglages::default()
            }
            .assainir();
            assert_eq!(r.themes.len(), EMPLACEMENTS_THEMES, "{combien}");
        }
    }

    #[test]
    fn un_emplacement_garde_ce_qu_on_y_enregistre() {
        let mut r = Reglages {
            accent: "#A06BFF".to_string(),
            fond_palette: "ember".to_string(),
            ..Reglages::default()
        };
        let garde = r.apparence();
        r.themes[1] = Theme {
            nom: "  Violet  ".to_string(),
            apparence: Some(garde.clone()),
        };

        // On change d'avis, puis on recharge l'emplacement.
        r.accent = "#FF6B4A".to_string();
        r.fond_palette = "slate".to_string();
        let r = r.assainir();
        assert_eq!(r.themes[1].nom, "Violet");

        let mut r = r;
        let repose = r.themes[1].apparence.clone().unwrap();
        r.appliquer_apparence(&repose);
        assert_eq!(r.accent, "#A06BFF");
        assert_eq!(r.fond_palette, "ember");
    }

    #[test]
    fn un_emplacement_abime_est_assaini_comme_le_reste() {
        // Ce qui dort dans un emplacement finit dans un attribut style
        // le jour ou on le recharge : il subit donc le meme controle.
        let r = Reglages {
            themes: vec![
                Theme {
                    nom: "x".repeat(200),
                    apparence: Some(Apparence {
                        accent: "#000;background:url(http://ailleurs)".to_string(),
                        fond_palette: "neon".to_string(),
                        ..Apparence::default()
                    }),
                },
                Theme::default(),
                Theme::default(),
            ],
            ..Reglages::default()
        }
        .assainir();
        let a = r.themes[0].apparence.clone().unwrap();
        assert_eq!(a.accent, ACCENT_DEFAUT);
        assert_eq!(a.fond_palette, "storm");
        assert_eq!(r.themes[0].nom.chars().count(), 40);
    }

    #[test]
    fn un_emplacement_vide_le_reste() {
        let r = Reglages::default().assainir();
        assert!(r.themes.iter().all(|t| t.apparence.is_none()));
        assert!(r.themes.iter().all(|t| t.nom.is_empty()));
    }

    #[test]
    fn l_apparence_fait_l_aller_retour_sans_rien_perdre() {
        let r = Reglages {
            accent: "#5B8CFF".to_string(),
            fond_palette: "custom".to_string(),
            fond_intensite: "vivid".to_string(),
            fond_vitesse: "fast".to_string(),
            fond_personnalise: [
                "#111111".to_string(),
                "#222222".to_string(),
                "#333333".to_string(),
            ],
            ..Reglages::default()
        };
        let a = r.apparence();
        let mut vierge = Reglages::default();
        vierge.appliquer_apparence(&a);
        assert_eq!(vierge.apparence(), a);
    }

    #[test]
    fn la_densite_par_defaut_est_equilibree() {
        assert_eq!(Reglages::default().densite, "equilibre");
    }

    #[test]
    fn une_densite_inconnue_revient_a_equilibre() {
        let r = Reglages {
            densite: "enorme".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.densite, "equilibre");
    }

    #[test]
    fn les_trois_densites_passent_intactes() {
        for densite in DENSITES {
            let r = Reglages {
                densite: densite.to_string(),
                ..Reglages::default()
            }
            .assainir();
            assert_eq!(r.densite, densite);
        }
    }

    #[test]
    fn le_filtre_mot_de_passe_dit_ce_que_le_filtre_attend() {
        let faire = |valeur: &str| Reglages {
            serveurs_mot_de_passe: valeur.to_string(),
            ..Reglages::default()
        };
        assert_eq!(faire("tous").prive_demande(), None);
        assert_eq!(faire("libres").prive_demande(), Some(false));
        assert_eq!(faire("prives").prive_demande(), Some(true));
        // Une valeur qui n'a pas ete assainie ne doit pas filtrer au
        // hasard : elle ne filtre rien.
        assert_eq!(faire("n_importe_quoi").prive_demande(), None);
    }

    #[test]
    fn un_filtre_mot_de_passe_inconnu_revient_a_tous() {
        let r = Reglages {
            serveurs_mot_de_passe: "peut-etre".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.serveurs_mot_de_passe, "tous");
    }

    #[test]
    fn le_sens_du_tri_par_defaut_est_celui_du_critere_par_defaut() {
        let r = Reglages::default();
        assert_eq!(
            r.serveurs_tri_croissant,
            crate::serveurs::ordre_naturel_croissant(r.serveurs_tri)
        );
        // Donc les plus peuples en tete.
        assert!(!r.serveurs_tri_croissant);
    }

    #[test]
    fn les_favoris_perdent_leurs_blancs_leurs_vides_et_leurs_doublons() {
        let r = Reglages {
            favoris: mots(&["  atria  ", "", "atria", "   ", "smokegun"]),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.favoris, mots(&["atria", "smokegun"]));
    }

    #[test]
    fn les_favoris_sont_bornes_en_nombre_et_en_longueur() {
        let longs: Vec<String> = (0..FAVORIS_MAX + 20)
            // L'index en tete : coupe a soixante, la fin disparait, et
            // deux logins qui ne se distinguaient que par leur fin
            // deviendraient le meme.
            .map(|i| format!("{i}-{}", "l".repeat(80)))
            .collect();
        let r = Reglages {
            favoris: longs,
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.favoris.len(), FAVORIS_MAX);
        assert!(r.favoris.iter().all(|f| f.chars().count() <= 60));
    }

    #[test]
    fn l_etoile_ajoute_puis_retire() {
        let mut favoris = Vec::new();
        assert!(basculer_favori(&mut favoris, "atria"));
        assert_eq!(favoris, mots(&["atria"]));
        assert!(!basculer_favori(&mut favoris, "atria"));
        assert!(favoris.is_empty());
    }

    #[test]
    fn l_etoile_ignore_un_login_vide_et_respecte_la_borne() {
        let mut favoris = Vec::new();
        assert!(!basculer_favori(&mut favoris, "   "));
        assert!(favoris.is_empty());

        let mut pleins: Vec<String> = (0..FAVORIS_MAX).map(|i| format!("s{i}")).collect();
        assert!(!basculer_favori(&mut pleins, "un_de_plus"));
        assert_eq!(pleins.len(), FAVORIS_MAX);
        // Retirer reste possible quand la liste est pleine, sinon on
        // serait coince.
        assert!(!basculer_favori(&mut pleins, "s0"));
        assert_eq!(pleins.len(), FAVORIS_MAX - 1);
    }

    #[test]
    fn les_nouveaux_champs_ne_cassent_pas_un_ancien_fichier() {
        // Un fichier ecrit avant le lot 19d n'a ni densite, ni favoris,
        // ni filtre mot de passe. Il doit s'ouvrir avec les defauts.
        let ancien = r##"{"langue":"fr","accent":"#20E3B2","assistant_fait":true}"##;
        let r: Reglages = serde_json::from_str(ancien).unwrap();
        let r = r.assainir();
        assert_eq!(r.langue, "fr");
        assert!(r.assistant_fait);
        assert_eq!(r.densite, "equilibre");
        assert_eq!(r.serveurs_mot_de_passe, "tous");
        assert!(!r.serveurs_favoris);
        assert!(r.favoris.is_empty());
    }

    #[test]
    fn les_couleurs_du_jeu_sont_rendues_par_defaut() {
        assert!(Reglages::default().couleurs_jeu);
        // Et un ancien fichier, qui n'a pas le champ, les rend aussi :
        // c'est le comportement d'avant le reglage.
        let ancien = r##"{"langue":"fr","accent":"#20E3B2"}"##;
        let r: Reglages = serde_json::from_str(ancien).unwrap();
        assert!(r.assainir().couleurs_jeu);
    }

    #[test]
    fn la_croix_pose_la_question_au_depart() {
        let r = Reglages::default();
        assert!(r.fermeture_demande());
        assert!(!r.fermeture_reduit());
        assert!(!r.demarrage_auto);
    }

    #[test]
    fn les_trois_fermetures_passent_et_le_reste_revient_a_demander() {
        for valeur in FERMETURES {
            let r = Reglages {
                fermeture: valeur.to_string(),
                ..Reglages::default()
            }
            .assainir();
            assert_eq!(r.fermeture, valeur);
        }
        let r = Reglages {
            fermeture: "exploser".to_string(),
            ..Reglages::default()
        }
        .assainir();
        assert_eq!(r.fermeture, "demander");
        // Une valeur abimee ne doit surtout pas devenir « quitter » :
        // quelqu'un perdrait sa session sans comprendre pourquoi.
        assert!(r.fermeture_demande());
    }

    #[test]
    fn un_ancien_fichier_garde_la_question_et_pas_de_demarrage_auto() {
        let ancien = r##"{"langue":"fr","accent":"#20E3B2"}"##;
        let r: Reglages = serde_json::from_str(ancien).unwrap();
        let r = r.assainir();
        assert_eq!(r.fermeture, "demander");
        assert!(!r.demarrage_auto);
    }

    fn ordre(liste: &[&str]) -> Vec<String> {
        liste.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn deplacer_une_tuile_vers_la_droite() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        // Lachee sur la moitie droite de la derniere : elle passe apres.
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "favorites", "browse", false),
            ordre(&["popular", "browse", "favorites"])
        );
        // Sur la moitie gauche de la meme : elle passe avant.
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "favorites", "browse", true),
            ordre(&["popular", "favorites", "browse"])
        );
    }

    #[test]
    fn deplacer_une_tuile_vers_la_gauche() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "browse", "favorites", true),
            ordre(&["browse", "favorites", "popular"])
        );
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "browse", "favorites", false),
            ordre(&["favorites", "browse", "popular"])
        );
    }

    #[test]
    fn lacher_une_tuile_sur_elle_meme_ne_change_rien() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        for avant in [true, false] {
            assert_eq!(
                deplacer_tuile(Grille::Jouer, &depart, "popular", "popular", avant),
                depart
            );
        }
    }

    #[test]
    fn un_deplacement_garde_toujours_les_trois_tuiles() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        for depuis in TUILES_JOUER {
            for vers in TUILES_JOUER {
                for avant in [true, false] {
                    let sortie = deplacer_tuile(Grille::Jouer, &depart, depuis, vers, avant);
                    assert_eq!(sortie.len(), TUILES_JOUER.len());
                    for tuile in TUILES_JOUER {
                        assert!(sortie.iter().any(|t| t == tuile), "{tuile} a disparu");
                    }
                }
            }
        }
    }

    #[test]
    fn les_deux_grilles_ne_se_melangent_pas() {
        // Le vrai risque de la generalisation : ranger l'accueil avec les
        // tuiles d'Editer, ou l'inverse. Une tuile etrangere a la grille
        // est jetee comme n'importe quelle tuile inconnue, et la grille
        // se recompose avec les siennes.
        let jouer = ordre(&["favorites", "popular", "browse"]);
        let editer = ordre(&["maps", "replays", "packs"]);

        assert_eq!(ordre_assaini(Grille::Jouer, &editer), jouer);
        assert_eq!(ordre_assaini(Grille::Editer, &jouer), editer);

        // Et un deplacement qui vise une tuile de l'autre grille ne fait
        // rien plutot que de la faire entrer.
        assert_eq!(
            glisser_sur(Grille::Editer, &editer, "maps", "popular"),
            editer
        );
        assert_eq!(glisser_sur(Grille::Jouer, &jouer, "maps", "popular"), jouer);
    }

    #[test]
    fn la_grille_d_editer_se_range_comme_celle_de_jouer() {
        let depart = ordre(&["maps", "replays", "packs"]);
        assert_eq!(
            glisser_sur(Grille::Editer, &depart, "maps", "packs"),
            ordre(&["replays", "packs", "maps"])
        );
        assert_eq!(
            decaler_tuile(Grille::Editer, &depart, "packs", -1),
            ordre(&["maps", "packs", "replays"])
        );
        // Au bord, rien ne bouge, comme sur l'autre grille.
        assert_eq!(decaler_tuile(Grille::Editer, &depart, "maps", -1), depart);
    }

    #[test]
    fn les_cles_de_grille_font_l_aller_et_le_retour() {
        for grille in [Grille::Jouer, Grille::Editer] {
            let cle = match grille {
                Grille::Jouer => "jouer",
                Grille::Editer => "editer",
            };
            assert_eq!(Grille::depuis_cle(cle), Some(grille));
        }
        for cle in ["", "Jouer", "play", "edit", "tuiles"] {
            assert_eq!(Grille::depuis_cle(cle), None, "{cle} aurait du etre refuse");
        }
    }

    #[test]
    fn glisser_sur_pose_la_tuile_sur_la_case_de_la_cible() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        // Vers la droite : la tuile portee prend la case de la cible,
        // la cible recule d'un cran.
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "favorites", "popular"),
            ordre(&["popular", "favorites", "browse"])
        );
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "favorites", "browse"),
            ordre(&["popular", "browse", "favorites"])
        );
        // Vers la gauche, le meme resultat dans l'autre sens.
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "browse", "popular"),
            ordre(&["favorites", "browse", "popular"])
        );
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "browse", "favorites"),
            ordre(&["browse", "favorites", "popular"])
        );
    }

    #[test]
    fn glisser_sur_met_vraiment_la_tuile_a_la_place_visee() {
        // La promesse tient en une ligne : apres le glisser, la tuile
        // portee occupe la place qu'occupait la cible. C'est ce qui
        // permet de montrer le resultat pendant le geste.
        let depart = ordre(&["favorites", "popular", "browse"]);
        for depuis in TUILES_JOUER {
            for vers in TUILES_JOUER {
                if depuis == vers {
                    continue;
                }
                let place = depart.iter().position(|t| t == vers).unwrap();
                let sortie = glisser_sur(Grille::Jouer, &depart, depuis, vers);
                assert_eq!(sortie[place], depuis, "{depuis} sur {vers}");
                assert_eq!(sortie.len(), TUILES_JOUER.len());
            }
        }
    }

    #[test]
    fn glisser_une_tuile_sur_elle_meme_ne_change_rien() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        for tuile in TUILES_JOUER {
            assert_eq!(glisser_sur(Grille::Jouer, &depart, tuile, tuile), depart);
        }
    }

    #[test]
    fn glisser_et_decaler_s_accordent_sur_les_voisines() {
        // Porter une tuile sur sa voisine revient a la pousser d'un cran.
        // Deux gestes, la souris et le clavier, et un seul resultat.
        let depart = ordre(&["favorites", "popular", "browse"]);
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "favorites", "popular"),
            decaler_tuile(Grille::Jouer, &depart, "favorites", 1)
        );
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "browse", "popular"),
            decaler_tuile(Grille::Jouer, &depart, "browse", -1)
        );
    }

    #[test]
    fn une_tuile_inconnue_ne_casse_rien() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "fantome", "browse", true),
            depart
        );
        assert_eq!(
            deplacer_tuile(Grille::Jouer, &depart, "favorites", "fantome", true),
            depart
        );
        assert_eq!(decaler_tuile(Grille::Jouer, &depart, "fantome", -1), depart);
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "fantome", "browse"),
            depart
        );
        assert_eq!(
            glisser_sur(Grille::Jouer, &depart, "favorites", "fantome"),
            depart
        );
    }

    #[test]
    fn decaler_echange_avec_la_voisine_et_s_arrete_aux_bords() {
        let depart = ordre(&["favorites", "popular", "browse"]);
        assert_eq!(
            decaler_tuile(Grille::Jouer, &depart, "popular", -1),
            ordre(&["popular", "favorites", "browse"])
        );
        assert_eq!(
            decaler_tuile(Grille::Jouer, &depart, "popular", 1),
            ordre(&["favorites", "browse", "popular"])
        );
        // Aux bords, rien ne bouge et surtout rien ne boucle.
        assert_eq!(
            decaler_tuile(Grille::Jouer, &depart, "favorites", -1),
            depart
        );
        assert_eq!(decaler_tuile(Grille::Jouer, &depart, "browse", 1), depart);
    }

    #[test]
    fn decaler_et_deplacer_s_accordent_sur_les_voisines() {
        // Deux chemins, un seul resultat quand les tuiles sont cote a
        // cote : c'est ce qui garantit que la souris et le clavier ne
        // divergent pas.
        let depart = ordre(&["favorites", "popular", "browse"]);
        assert_eq!(
            decaler_tuile(Grille::Jouer, &depart, "favorites", 1),
            deplacer_tuile(Grille::Jouer, &depart, "favorites", "popular", false)
        );
        assert_eq!(
            decaler_tuile(Grille::Jouer, &depart, "browse", -1),
            deplacer_tuile(Grille::Jouer, &depart, "browse", "popular", true)
        );
    }
}
