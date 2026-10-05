// Les constantes de presentation, reprises telles quelles de la maquette.
//
// Elles ne vivent pas dans `cairn-core` parce qu'elles ne decident de
// rien : ce sont des valeurs d'affichage. Ce qui est une regle, comme
// l'ordre des tuiles ou le domaine d'une palette, est en Rust et teste.

export type Palette = {
  label: string;
  dots: [string, string, string];
  alpha: [number, number, number];
};

export const PALETTES: Record<string, Palette> = {
  storm: { label: 'Storm', dots: ['#20E3B2', '#5B8CFF', '#FF6B4A'], alpha: [0.85, 0.62, 0.38] },
  aurora: { label: 'Aurora', dots: ['#3DDC84', '#33C9FF', '#A06BFF'], alpha: [0.8, 0.6, 0.42] },
  ember: { label: 'Ember', dots: ['#FF8A3D', '#FF4D6D', '#FFC857'], alpha: [0.72, 0.52, 0.38] },
  slate: { label: 'Slate', dots: ['#8FA0B5', '#6E7FA0', '#B0BCCB'], alpha: [0.62, 0.48, 0.34] },
};

export const INTENSITES: Record<string, { op: number; blur: string; glow: number }> = {
  soft: { op: 0.4, blur: '64px', glow: 0.07 },
  medium: { op: 0.6, blur: '54px', glow: 0.11 },
  vivid: { op: 0.86, blur: '44px', glow: 0.16 },
};

export const VITESSES: Record<string, number> = { slow: 1.7, normal: 1, fast: 0.55 };

/** Duree de base des trois halos, en secondes, avant multiplication. */
export const DUREES = [58, 76, 66];

export const ACCENTS = [
  { name: 'Storm teal', color: '#20E3B2' },
  { name: 'Blue', color: '#5B8CFF' },
  { name: 'Coral', color: '#FF6B4A' },
  { name: 'Amber', color: '#FFC857' },
  { name: 'Violet', color: '#A06BFF' },
  { name: 'Rose', color: '#FF4D6D' },
];

/** Duree d'une slide du carrousel des actualites, en secondes. */
export const SLIDE_S = 10;

/**
 * Les deux emplacements d'installation les plus courants.
 *
 * Chez Flow c'est le premier, chez beaucoup de joueurs le second,
 * ManiaPlanet ayant ete distribue par Steam. Ils ne servent que de
 * suggestion dans l'assistant : le chemin reel vient du registre, lu par
 * Rust.
 */
export const CHEMINS_COURANTS = [
  {
    label: 'Standalone install',
    path: 'C:\\Program Files (x86)\\ManiaPlanet\\ManiaPlanet.exe',
  },
  {
    label: 'Steam install',
    path: 'C:\\Program Files (x86)\\Steam\\steamapps\\common\\ManiaPlanet_SMStorm\\ManiaPlanet.exe',
  },
];

/**
 * Les onglets du bandeau, dans l'ordre ou ils se parcourent.
 *
 * A ramene vers la gauche, E pousse vers la droite. Les rappels clavier
 * tiennent donc dans les onglets qui sont au bout de la course, et le
 * raccourci se lit a l'endroit ou il mene.
 *
 * **Reglages n'est pas un onglet.** La maquette en avait un, qui menait
 * aux reglages du JEU, pendant que la roue dentee menait aux reglages du
 * LAUNCHER : deux mots proches pour deux destinations differentes,
 * confusion garantie. Et surtout, une place d'onglet principal etait
 * depensee pour un ecran qui ne fait qu'une chose, ouvrir le jeu. Cet
 * ecran est descendu dans Reglages, section Jeu, ou l'utilisateur ira le
 * chercher puisque chercher un reglage derriere une roue dentee est un
 * reflexe acquis.
 *
 * **Cinq onglets depuis l'etape 30**, et Jouer est au milieu. Ce n'est
 * pas de la symetrie pour la symetrie : c'est l'ecran d'ou l'on part et
 * ou l'on revient, donc il est a la distance la plus courte des deux
 * bords, dans les deux sens.
 *
 * **Parcourir les serveurs devient un onglet.** C'etait un sous-ecran de
 * Jouer, avec sa propre fleche de retour et sans bandeau. Il n'y avait
 * aucune raison a ca : on y passe autant de temps que sur l'accueil, et
 * un ecran qui cache le bandeau coupe la navigation le temps qu'on y est.
 * La tuile Parcourir de l'accueil y mene toujours, elle change juste
 * d'onglet au lieu d'ouvrir un sous-ecran.
 *
 * Classement et Historique arrivent vides, avec leur carte « a venir ».
 * Ils se remplissent a l'etape 32 de la phase 4.
 */
export const ONGLETS = [
  // `touche` nomme le SENS et pas la lettre : l'etiquette affichee vient
  // du systeme, elle vaut A sur un clavier francais et Q sur un anglais.
  { id: 'community', libelle: 'Community', touche: 'precedent' },
  { id: 'servers', libelle: 'Servers', touche: '' },
  { id: 'play', libelle: 'Play', touche: '' },
  { id: 'leaderboard', libelle: 'Leaderboard', touche: '' },
  { id: 'history', libelle: 'History', touche: 'suivant' },
] as const;

/** L'ecran d'accueil, celui ou mene le bouton Accueil du bandeau. */
export const ECRAN_ACCUEIL = 'play';

/** Le seul ordre de reference, pour la navigation comme pour l'affichage. */
export const ORDRE_ONGLETS: string[] = ONGLETS.map((o) => o.id);

/**
 * Les cinq paliers de rang, leurs noms et leurs couleurs.
 *
 * Les couleurs ne sont pas des metaux photographies : ce sont des
 * teintes choisies pour se distinguer les unes des autres **en
 * peripherie du regard**, sur le fond sombre de Cairn. Dans une colonne
 * qu'on balaye, un joueur doit reconnaitre un palier sans lire son nom,
 * et un bronze et un or trop fideles au metal se ressemblent des qu'ils
 * sont petits.
 *
 * L'ordre est celui de `cairn_core::rang::Palier`, indice compris : les
 * deux listes se repondent et ne doivent pas diverger.
 */
export const PALIERS = [
  { id: 'bronze', libelle: 'Bronze', couleur: '#C08457' },
  { id: 'argent', libelle: 'Silver', couleur: '#AEB9C4' },
  { id: 'or', libelle: 'Gold', couleur: '#E8B93E' },
  { id: 'platine', libelle: 'Platinum', couleur: '#5FD4D0' },
  { id: 'diamant', libelle: 'Diamond', couleur: '#9B8CFF' },
] as const;

/** Les points qu'il faut traverser pour changer de palier. */
export const POINTS_PAR_PALIER = 100;

/**
 * Les deux couleurs d'un resultat de match.
 *
 * Elles sont ici et non dans une feuille de style parce que trois ecrans
 * vont s'en servir, l'Historique, le profil et le tableau de fin de
 * partie, et qu'un vert de victoire different d'un ecran a l'autre est
 * exactement le genre de detail qui fait douter de ce qu'on lit.
 *
 * Elles ne suivent pas l'accent, volontairement : l'accent est un gout,
 * une victoire est un fait. Un joueur qui met son accent en rouge ne doit
 * pas voir ses victoires en rouge.
 */
export const VICTOIRE = '#2ED3A0';
export const DEFAITE = '#E3706A';

/**
 * Les categories d'actualites, leurs libelles et leurs couleurs.
 *
 * `autre` ferme la liste et recoit tout ce que Cairn ne connait pas :
 * une categorie publiee plus tard ne doit pas faire disparaitre
 * l'article chez ceux qui n'ont pas mis a jour.
 */
export const CATEGORIES = [
  { id: 'tournoi', libelle: 'Tournaments', couleur: '#FFC857' },
  { id: 'maps', libelle: 'Maps', couleur: '#5FD4D0' },
  { id: 'creation', libelle: 'Creation', couleur: '#9B8CFF' },
  { id: 'technique', libelle: 'Technical', couleur: '#80D5FF' },
  { id: 'entraide', libelle: 'Help', couleur: '#2ED3A0' },
  { id: 'autre', libelle: 'Other', couleur: '#838C96' },
] as const;

export type Tuile = {
  id: string;
  label: string;
  icone: string;
  /** `null` veut dire « prend la couleur d'accent ». */
  couleur: string | null;
};

/** Les tuiles de l'ecran Jouer. */
export const TUILES: Record<string, Tuile> = {
  favorites: { id: 'favorites', label: 'Favorites', icone: 'star', couleur: '#FFC857' },
  popular: { id: 'popular', label: 'Popular server', icone: 'flame', couleur: '#FF6B4A' },
  browse: { id: 'browse', label: 'Browse servers', icone: 'grid', couleur: null },
};

/**
 * Celles de l'ecran Editer, et leurs noms disent pourquoi elles ont change.
 *
 * La maquette portait « Create a map », « Edit a replay » et « Title
 * tools », trois promesses d'ouvrir un editeur. Le jeu ne sait pas le
 * faire : aucun drapeau de ligne de commande ni aucune commande du
 * protocole `maniaplanet://` n'ouvre l'editeur de maps, celui de
 * replays ou les outils de titre. Verifie sur la documentation
 * officielle au lot 22d, pas suppose.
 *
 * Les tuiles disent donc ce que Cairn sait vraiment faire, et que les
 * menus du jeu font mal : montrer ce qu'on a. Combien de maps, quand
 * pour la derniere fois, ou elles sont, et quels titres sont installes.
 * Le bouton qui ouvre le jeu reste, sur le menu du bon titre.
 *
 * Les identifiants doivent rester ceux de `cairn_core::reglages::TUILES_EDITER`
 * et ceux de `inventaire::Sorte`, les trois listes se repondent.
 */
export const TUILES_EDITER: Record<string, Tuile> = {
  maps: { id: 'maps', label: 'My maps', icone: 'create', couleur: '#6FA8FF' },
  replays: { id: 'replays', label: 'My replays', icone: 'replay', couleur: '#C08BFF' },
  packs: { id: 'packs', label: 'Title packs', icone: 'tools', couleur: null },
};
