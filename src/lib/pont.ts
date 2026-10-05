// Le pont vers Rust.
//
// Tous les appels a `invoke` passent par ici, et nulle part ailleurs.
// Deux raisons. La premiere est qu'on veut un seul endroit ou lire le
// contrat entre l'interface et le coeur. La seconde est que les types
// ci dessous doivent suivre les structures Rust : le jour ou l'une bouge,
// c'est ce fichier qui doit faire echouer `svelte-check`, pas un
// composant au hasard.
//
// Les noms de champs sont ceux que serde produit, donc en francais et en
// serpent. On ne les renomme pas au passage : un champ qui change de nom
// entre Rust et l'interface est un bogue qui attend son heure.

import { invoke } from '@tauri-apps/api/core';

export type Diagnostic = {
  nom: string;
  version: string;
  coeur: string;
  /**
   * Le numero de contrat du moteur, voir `cairn_core::CONTRAT`.
   *
   * Facultatif dans le type, et c'est volontaire : un moteur d'avant ce
   * mecanisme ne le rend pas, et c'est exactement le cas qu'on veut
   * detecter.
   */
  contrat?: number;
  os: string;
  arch: string;
  journal: string | null;
};

/**
 * Le numero de contrat que CETTE interface attend.
 *
 * A garder egal a `cairn_core::CONTRAT`. Les deux bougent ensemble le
 * jour ou un champ ou une commande du pont change de forme, et c'est
 * leur desaccord qui revele un binaire perime.
 */
export const CONTRAT_ATTENDU = 7;

/** Laquelle des deux grilles de tuiles on range. */
export type Grille = 'jouer' | 'editer';

/** Les trois sortes de fichiers que l'ecran Editer resume. */
export type SorteFichiers = 'maps' | 'replays' | 'packs';

/** Un fichier de l'inventaire, pret a afficher. */
export type ElementFichier = {
  /** Le nom sans son extension, qui est ce que le joueur a tape. */
  nom: string;
  /** Le sous-dossier qui le porte, vide quand il est a la racine. */
  groupe: string;
  octets: number;
  /** Secondes depuis 1970. */
  modifie: number;
  /**
   * L'identifiant de titre, pour un title pack et pour lui seul.
   *
   * `null` quand le nom du fichier n'en porte pas un valable, ce qui
   * arrive des qu'il a ete renomme a la main. La tuile affiche alors le
   * pack sans son bouton d'ouverture.
   */
  titre: string | null;
};

/** Un sous-dossier et ce qu'il pese. */
export type GroupeFichiers = { nom: string; nombre: number; octets: number };

/** Le resume d'une sorte de fichiers. */
export type Inventaire = {
  nombre: number;
  octets: number;
  /** La date du plus recent, ou `null` quand il n'y a aucun fichier. */
  dernier: number | null;
  groupes: GroupeFichiers[];
  recents: ElementFichier[];
};

export type Profil = {
  id: string;
  nom: string;
  /** Date au format AAAA-MM-JJ, la phrase se compose dans l'interface. */
  derniere_utilisation: string | null;
  /** Couleur de la pastille, tiree du nom donc stable dans le temps. */
  teinte: string;
  /** Lettre affichee dans la pastille. */
  initiale: string;
};

/**
 * Ce que le coup d'oeil des cinq secondes rapporte.
 *
 * Voir `cairn_core::veille` : la boucle ne redemande plus l'etat
 * complet, elle regarde ce qui bouge et ne reveille le reste que quand
 * quelque chose a bouge.
 */
export type Veille = {
  jeu_lance: boolean;
  profils_a_relire: boolean;
  jeu_a_change: boolean;
};

/**
 * L'historique de navigation, vu de l'interface.
 *
 * Les deux drapeaux servent a eteindre une fleche plutot qu'a la faire
 * disparaitre : une commande qui change de place est plus derangeante
 * qu'une commande eteinte.
 */
export type EtatNavigation = {
  ecran: string;
  peut_reculer: boolean;
  peut_avancer: boolean;
};

export type Etat = {
  chemin_jeu: string | null;
  jeu_lance: boolean;
  profils: Profil[];
  profil_actif: string | null;
};

/**
 * Un morceau de nom venu du jeu, pret a afficher.
 *
 * La couleur est deja ramenee a un contraste lisible sur notre fond
 * sombre, voir `cairn_core::lisibilite`. `null` veut dire que le
 * morceau prend la couleur ordinaire de l'interface.
 */
export type SegmentAffiche = {
  texte: string;
  couleur: string | null;
  gras: boolean;
  italique: boolean;
};

export type Serveur = {
  nom: string;
  nom_brut: string;
  /** Le nom decoupe et colore. C'est ce qu'on affiche. */
  nom_segments: SegmentAffiche[];
  mode: string;
  login: string;
  titre: string;
  joueurs: number;
  maximum: number;
  observateurs: number;
  latence: number | null;
  description: string;
  /** Le pays, tire de la zone. Vide quand la zone ne le dit pas. */
  region: string;
  zone: string;
  /** Demande un mot de passe : c'est le cadenas de la liste. */
  prive: boolean;
  observation_privee: boolean;
  /** Porte d'entree du matchmaking. */
  lobby: boolean;
  /** Le login du serveur dont celui-ci n'est qu'un relais. */
  relais: string | null;
  /** Les bornes de ladder, quand le serveur en impose. */
  ladder: [number, number] | null;
};

export type Tri = 'joueurs' | 'nom' | 'mode';

/** Le filtre mot de passe : tous, ceux qui n'en demandent pas, les autres. */
export type MotDePasse = 'tous' | 'libres' | 'prives';

/** L'espacement general de l'ecran Jouer. */
export type Densite = 'compact' | 'equilibre' | 'aere';

/** Ce que fait la croix de la fenetre. */
export type Fermeture = 'demander' | 'reduire' | 'quitter';

export type ModeCompte = { mode: string; compte: number };

export type Vue = {
  peuples: Serveur[];
  vides: Serveur[];
  comptes: ModeCompte[];
  total: number;
  retenus: number;
};

export type Apparence = {
  accent: string;
  fond_palette: string;
  fond_intensite: string;
  fond_vitesse: string;
  fond_personnalise: [string, string, string];
};

export type Theme = {
  nom: string;
  /** `null` quand l'emplacement est vide. */
  apparence: Apparence | null;
};

export type Reglages = {
  langue: 'fr' | 'en';
  accent: string;
  fond_palette: string;
  fond_intensite: string;
  fond_vitesse: string;
  fond_personnalise: [string, string, string];
  animations: boolean;
  /** Ce que fait la croix de la fenetre. */
  fermeture: Fermeture;
  /**
   * Cairn s'ouvre avec la session Windows.
   *
   * La verite est dans le registre. Ce champ n'en est qu'un reflet, que
   * Rust remet en accord au demarrage et apres chaque enregistrement.
   */
  demarrage_auto: boolean;
  /** Rendre les couleurs et les styles que le jeu met dans les noms. */
  couleurs_jeu: boolean;
  /** L'espacement general, voir la section Apparence des reglages. */
  densite: Densite;
  ordre_tuiles: string[];
  /** Celui de l'ecran Editer, depuis le lot 22d. */
  ordre_tuiles_editer: string[];
  profil: string | null;
  /** Le chemin de ManiaPlanet.exe designe a la main, s'il y en a un. */
  chemin_jeu: string | null;
  assistant_fait: boolean;
  /** La touche qui ramene a gauche, par sa position et non sa lettre. */
  touche_precedent: string;
  touche_suivant: string;
  souris_laterale: boolean;
  themes: Theme[];
  /** Le mode retenu dans la barre de filtres. Vide veut dire tous. */
  serveurs_mode: string;
  serveurs_tri: Tri;
  serveurs_peuples_ouverts: boolean;
  serveurs_vides_ouverts: boolean;
  /** Le sens du tri. Chaque critere a le sien au premier clic. */
  serveurs_tri_croissant: boolean;
  serveurs_mot_de_passe: MotDePasse;
  /** Ne montrer que les favoris. */
  serveurs_favoris: boolean;
  /** Les serveurs mis en favori, par leur login. */
  favoris: string[];
};

export type Rejointe = { relais: boolean };

/**
 * Vrai quand l'interface tourne dans la fenetre Tauri.
 *
 * Ouverte dans un navigateur, pour une verification de rendu, il n'y a
 * pas de pont. Le test evite une cascade d'appels voues a l'echec, et il
 * porte sur l'objet que Tauri installe, pas sur le nom de l'hote : la
 * maquette du lot 18 testait `location.hostname`, ce qui ne veut plus
 * rien dire ici puisque la page est servie par l'application elle meme.
 */
export function pontPresent(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export const diagnostic = () => invoke<Diagnostic>('diagnostic');

export const etat = (profilMemorise: string | null) =>
  invoke<Etat>('etat', { profilMemorise });

/** Le coup d'oeil des cinq secondes, voir `Veille`. */
export const veille = () => invoke<Veille>('veille');

export const serveurs = () => invoke<Serveur[]>('serveurs');

export const vueServeurs = (
  mode: string,
  recherche: string,
  tri: Tri,
  croissant: boolean,
  motDePasse: MotDePasse,
  favorisSeuls: boolean,
) =>
  invoke<Vue>('vue_serveurs', {
    mode: mode || null,
    recherche,
    tri,
    croissant,
    motDePasse,
    favorisSeuls,
  });

/**
 * Ce que devient le tri quand on clique sur un critere.
 *
 * Trois etats : le sens naturel, le sens inverse, puis le retour au tri
 * par defaut. Demande a Rust plutot que recopie ici : la regle est testee
 * la bas, et une regle ecrite dans deux langages finit par diverger.
 */
export const triSuivant = (courant: Tri, croissant: boolean, demande: Tri) =>
  invoke<[Tri, boolean]>('tri_suivant', { courant, croissant, demande });

/**
 * Va sur un ecran, et l'empile.
 *
 * Toutes les navigations passent par la, d'ou qu'elles viennent : un
 * onglet clique, les deux touches, la roue dentee, une tuile. C'est ce
 * qui permet a Precedent de remonter le chemin reellement parcouru.
 */
export const naviguer = (ecran: string) =>
  invoke<EtatNavigation>('naviguer', { ecran });

export const reculer = () => invoke<EtatNavigation>('reculer');

export const avancer = () => invoke<EtatNavigation>('avancer');

/** Met ou enleve un serveur des favoris. Rend les reglages a jour. */
export const basculerFavori = (login: string) =>
  invoke<Reglages>('basculer_favori', { login });

export const rejoindre = (
  login: string,
  titre: string,
  profil: string,
  spectate: boolean,
) => invoke<Rejointe>('rejoindre', { login, titre, profil, spectate });

export const ouvrirJeu = (profil: string) => invoke<Rejointe>('ouvrir_jeu', { profil });

export const reglages = () => invoke<Reglages>('reglages');

/** Rend les reglages tels que Rust les a retenus, assainis. */
export const enregistrerReglages = (valeurs: Reglages) =>
  invoke<Reglages>('enregistrer_reglages', { valeurs });

/**
 * Une tuile portee au dessus d'une autre, pendant un glisser.
 *
 * Elle prend la place de celle qu'elle survole. Pas de cote a deviner
 * ici : la grille se reorganise sous le
 * curseur, donc ce qu'on voit pendant le geste est deja le resultat.
 */
export const glisserTuile = (
  grille: Grille,
  ordre: string[],
  depuis: string,
  vers: string,
) => invoke<string[]>('glisser_tuile', { grille, ordre, depuis, vers });

/** Une tuile poussee d'un cran, pour les fleches et le clavier. */
export const decalerTuile = (grille: Grille, ordre: string[], id: string, pas: number) =>
  invoke<string[]>('decaler_tuile', { grille, ordre, id, pas });

/**
 * L'inventaire des fichiers du joueur, pour l'ecran Editer.
 *
 * Trois lectures de dossier cote Rust, sans ouvrir le moindre fichier.
 * Demandee a l'ouverture de l'ecran et sur clic du bouton de
 * rafraichissement, jamais dans la boucle de veille.
 */
export const inventaire = () =>
  invoke<Record<string, Inventaire>>('inventaire');

/**
 * Ouvre l'un des trois dossiers du jeu dans l'explorateur.
 *
 * Une cle, jamais un chemin. Aucun chemin du disque ne traverse le pont
 * dans ce sens, donc il n'y a rien a composer et rien a contourner : les
 * seuls dossiers qui puissent s'ouvrir sont ceux que Rust connait.
 */
export const ouvrirDossier = (cle: SorteFichiers) =>
  invoke<void>('ouvrir_dossier', { cle });

/** Ouvre le jeu sur le menu d'un title pack. */
export const ouvrirTitre = (titre: string, profil: string) =>
  invoke<Rejointe>('ouvrir_titre', { titre, profil });

/** Le genre d'une actualite, voir `cairn_core::actualites`. */
export type CategorieActu = 'tournoi' | 'maps' | 'creation' | 'technique' | 'entraide' | 'autre';

/**
 * Un morceau de texte dans un paragraphe d'article.
 *
 * **Aucun de ces champs ne contient de HTML**, et c'est la decision de
 * securite du lot 25 : Rust rend une structure, l'interface dessine de
 * vrais noeuds. Il n'y a donc nulle part de `{@html}`, donc rien a
 * assainir et rien a oublier d'assainir.
 */
export type Fragment =
  | { sorte: 'texte'; texte: string }
  | { sorte: 'gras'; texte: string }
  | { sorte: 'italique'; texte: string }
  | { sorte: 'code'; texte: string }
  | { sorte: 'lien'; texte: string; lien: string }
  | { sorte: 'image'; texte: string; lien: string };

/** Un bloc d'article. */
export type Bloc =
  | { sorte: 'titre'; niveau: number; texte: string }
  | { sorte: 'paragraphe'; fragments: Fragment[] }
  | { sorte: 'liste'; ordonnee: boolean; points: Fragment[][] }
  | { sorte: 'citation'; fragments: Fragment[] }
  | { sorte: 'code'; texte: string }
  | { sorte: 'separateur' };

export type Article = {
  id: string;
  titre: string;
  categorie: CategorieActu;
  /** Secondes depuis 1970. */
  date: number;
  resume: string;
  langue: string;
  epingle: boolean;
  corps: Bloc[];
};

/**
 * Ce qui empeche le fil d'etre frais.
 *
 * **Une cle et pas une phrase.** Rust ne parle qu'une langue, l'interface
 * se lit dans deux : une erreur rendue toute faite depuis Rust arriverait
 * en francais sur un ecran anglais. Le detail technique reste dans le
 * journal. Voir `cairn_core::actualites::Souci`.
 */
export type SouciActu =
  | 'reseau'
  | 'signature'
  | 'illisible'
  | 'rejeu'
  | 'tropgros'
  /** L'adresse du fil n'est pas encore renseignee. Ce n'est pas une panne. */
  | 'nonconfigure';

export type VueActualites = {
  articles: Article[];
  /** Les comptes par categorie, sur le fil entier et pas sur le filtre. */
  comptes: Array<[CategorieActu, number]>;
  publie: number | null;
  /**
   * Pourquoi le fil affiche n'est pas frais, quand c'est le cas.
   *
   * Il peut y avoir des articles ET un avertissement : un cache valable
   * avec un reseau coupe n'est pas une panne.
   */
  avertissement: SouciActu | null;
};

/**
 * Le fil d'actualites, verifie, trie et filtre.
 *
 * `forcer` demande d'aller voir le reseau meme si le cache est bon.
 */
export const actualites = (
  forcer: boolean,
  langue: string,
  categories: CategorieActu[],
  recherche: string,
) => invoke<VueActualites>('actualites', { forcer, langue, categories, recherche });

/**
 * Ouvre un lien d'article dans le navigateur du systeme.
 *
 * Rust revalide l'adresse avant de l'ouvrir : voir
 * `cairn_core::actualites::lien_autorise`. L'interface ne decide pas de
 * ce qui est ouvrable.
 */
export const ouvrirLien = (lien: string) => invoke<void>('ouvrir_lien', { lien });

/** Les trois niveaux d'une ligne de journal, voir `cairn_core::journal`. */
export type NiveauJournal = 'info' | 'alerte' | 'erreur';

/** Une ligne du journal, deja decoupee et classee par Rust. */
export type LigneJournal = {
  /**
   * Son rang dans le fichier, a partir de 1.
   *
   * Il ne se recalcule pas apres un filtre, volontairement : « la ligne
   * 214 » doit designer la meme ligne dans les deux vues, sinon deux
   * personnes qui regardent le meme journal ne parlent pas de la meme
   * chose.
   */
  numero: number;
  /** Vide sur une ligne orpheline, quand le fichier commence au milieu. */
  horodatage: string;
  niveau: NiveauJournal;
  message: string;
};

/**
 * Les comptes **du fichier entier**, et pas de ce qui est affiche.
 *
 * Un compte qui tombe a zero des qu'on filtre sur Info ne dit plus rien
 * de ce que contient le journal, alors que c'est justement sa raison
 * d'etre : annoncer qu'il y a trois erreurs la-dedans.
 */
export type ComptesJournal = {
  info: number;
  alerte: number;
  erreur: number;
};

export type VueJournal = {
  lignes: LigneJournal[];
  comptes: ComptesJournal;
  anonyme: boolean;
  /**
   * Combien de lignes retenues par le filtre manquent AVANT la premiere
   * affichee, parce que la limite les a ecartees.
   */
  tronquees: number;
};

/**
 * Lit le journal de Cairn, filtre et compte.
 *
 * **Aucun chemin ne traverse le pont**, comme pour `ouvrirDossier` : on
 * envoie des niveaux, une recherche et un choix, Cairn lit son propre
 * journal et lui seul.
 *
 * Le filtrage se fait en Rust et pas ici, pour la meme raison que la
 * recherche de serveurs : ce qui merite un test vit dans `cairn-core`.
 */
export const lireJournal = (
  niveaux: NiveauJournal[],
  recherche: string,
  anonyme: boolean,
  limite: number,
) => invoke<VueJournal>('lire_journal', { niveaux, recherche, anonyme, limite });

/**
 * Le rapport a envoyer, **toujours anonymise**, quel que soit l'affichage.
 *
 * Il porte le journal entier et pas la vue filtree : un rapport ou le
 * declarant a deja ecarte ce qu'il croyait sans rapport est exactement
 * celui ou manque la ligne utile.
 */
export const rapportJournal = () => invoke<string>('rapport_journal');

/** Ecrit le rapport a cote du journal et rend son nom de fichier. */
export const enregistrerRapport = () => invoke<string>('enregistrer_rapport');

/** Ouvre le dossier du journal dans l'explorateur. */
export const ouvrirDossierJournal = () => invoke<void>('ouvrir_dossier_journal');

export const quitter = () => invoke<void>('quitter');

/**
 * La reponse a la question posee par la croix.
 *
 * `memoriser` vient d'une case decochee par defaut, voir
 * `composants/Fermeture.svelte`.
 */
export const fermerOuReduire = (choix: 'reduire' | 'quitter', memoriser: boolean) =>
  invoke<void>('fermer_ou_reduire', { choix, memoriser });
