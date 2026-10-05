// L'etat de l'application.
//
// La maquette tenait tout dans un objet `S` et redessinait la page entiere
// a chaque changement, puis recollait le resultat noeud par noeud avec une
// fonction de rapiecage maison. Ces quatre-vingts lignes disparaissent :
// c'est exactement le travail de Svelte, en mieux teste.
//
// Ce qui reste est la separation qui comptait deja dans `S`, rendue
// explicite : d'un cote ce que Cairn retient d'une session a l'autre, de
// l'autre ce qui ne vit que le temps de la fenetre ouverte.

import * as pont from './pont';
import { fixerLangue } from './traduction.svelte';
import { chargerDisposition } from './clavier.svelte';
import { PALETTES, INTENSITES, VITESSES, DUREES } from './constantes';
import { rgba } from './couleurs';

/** Ce que Cairn retient. Miroir exact de `cairn_core::reglages`. */
export const reglages = $state<pont.Reglages>({
  langue: 'en',
  accent: '#20E3B2',
  fond_palette: 'storm',
  fond_intensite: 'medium',
  fond_vitesse: 'slow',
  fond_personnalise: ['#20E3B2', '#5B8CFF', '#FF6B4A'],
  animations: true,
  fermeture: 'demander',
  demarrage_auto: false,
  couleurs_jeu: true,
  densite: 'equilibre',
  ordre_tuiles: ['favorites', 'popular', 'browse'],
  ordre_tuiles_editer: ['maps', 'replays', 'packs'],
  profil: null,
  chemin_jeu: null,
  assistant_fait: false,
  touche_precedent: 'KeyQ',
  touche_suivant: 'KeyE',
  souris_laterale: true,
  themes: [
    { nom: '', apparence: null },
    { nom: '', apparence: null },
    { nom: '', apparence: null },
  ],
  serveurs_mode: '',
  serveurs_tri: 'joueurs',
  serveurs_peuples_ouverts: true,
  serveurs_vides_ouverts: false,
  serveurs_tri_croissant: false,
  serveurs_mot_de_passe: 'tous',
  serveurs_favoris: false,
  favoris: [],
});

/** Ce qui ne survit pas a la fermeture de la fenetre. */
export const ui = $state({
  /** La tuile d'Editer dont la fiche est ouverte, une seule a la fois. */
  editerOuvert: null as string | null,
  /** `play`, `edit`, `gamesettings`, et plus tard les autres. */
  ecran: 'play',
  /** L'assistant de premier lancement, et son etape. */
  assistant: false,
  etape: 1,
  /** Le selecteur de profil, en surimpression. */
  selecteurProfil: false,
  /** Le profil selectionne dans l'etape 2 de l'assistant. */
  choix: '' as string,
  /**
   * Un raccourci est en cours de capture dans les reglages.
   *
   * La navigation clavier se tait pendant ce temps : sans ca, appuyer
   * sur la touche qu'on veut affecter changerait d'onglet au moment meme
   * ou on essaie de la capturer.
   */
  captureTouche: false,
  /** La croix a ete cliquee et le reglage dit de poser la question. */
  fermetureDemandee: false,
  slide: 0,
  slideEnPause: false,
  popularOuvert: false,
  popularDetails: false,
  message: null as string | null,
  /**
   * Le rebond de la rangee d'onglets quand la navigation bute.
   *
   * Un compteur et pas un drapeau : c'est son incrementation qui
   * declenche l'animation, donc deux appuis de suite la rejouent. Un
   * drapeau demanderait de le remettre a zero entre les deux, ce qui ne
   * peut se faire qu'a l'image suivante et ne se fait pas du tout dans
   * un onglet ralenti par le systeme.
   */
  rebond: 0,
  rebondCote: 'gauche' as 'gauche' | 'droite',
  /**
   * L'historique a-t-il quelque chose derriere, devant.
   *
   * La verite est cote Rust, voir `cairn_core::historique`. Ces deux
   * drapeaux ne sont la que pour eteindre la fleche du bandeau.
   */
  peutReculer: false,
  peutAvancer: false,
  aide: true,
  pret: false,
});

/** Ce que le systeme repond. Relu, jamais devine. */
export const systeme = $state({
  cheminJeu: null as string | null,
  jeuLance: false,
  profils: [] as pont.Profil[],
  serveurs: [] as pont.Serveur[],
  serveursCharges: false,
  /** L'inventaire des fichiers, par sorte. Vide tant qu'on n'a pas lu. */
  inventaire: {} as Record<string, pont.Inventaire>,
  inventaireCharge: false,
  inventaireEnCours: false,
  erreur: null as string | null,
  /**
   * Vrai quand le moteur et l'interface ne sont pas de la meme version.
   *
   * Voir `cairn_core::CONTRAT`. Ce n'est pas une erreur au sens ou
   * quelque chose aurait echoue : l'application tourne, elle affiche
   * simplement moins que ce qu'elle sait afficher. D'ou un message a
   * part, et pas la banniere rouge des pannes.
   */
  contratDepareille: false,
  /**
   * Ce que le moteur dit de lui meme au demarrage.
   *
   * Garde en entier plutot que reduit au seul numero de contrat : l'ecran
   * Developpeur affiche la version, le systeme et l'architecture, et
   * redemander tout ca a Rust pour les afficher alors qu'on vient de les
   * recevoir n'aurait pas de sens.
   */
  diagnostic: null as pont.Diagnostic | null,
});

// ------------------------------------------------------------------
//  Le fond
// ------------------------------------------------------------------

/**
 * Les variables CSS du fond anime, recalculees quand un reglage bouge.
 *
 * C'est la seule chose de la maquette qui devait etre reconstruite a
 * chaque rendu et qui le reste, parce qu'elle est bel et bien derivee.
 */
export function variablesFond(): string {
  const palette = PALETTES[reglages.fond_palette];
  const teintes =
    reglages.fond_palette === 'custom' || !palette
      ? ([0.85, 0.62, 0.38] as const).map((a, i) => rgba(reglages.fond_personnalise[i], a))
      : palette.dots.map((d, i) => rgba(d, palette.alpha[i]));

  const intensite = INTENSITES[reglages.fond_intensite] ?? INTENSITES.medium;
  const facteur = VITESSES[reglages.fond_vitesse] ?? VITESSES.slow;

  return [
    `--w1:${teintes[0]}`,
    `--w2:${teintes[1]}`,
    `--w3:${teintes[2]}`,
    `--wave-op:${intensite.op}`,
    `--wave-blur:${intensite.blur}`,
    ...DUREES.map((d, i) => `--d${i + 1}:${(d * facteur).toFixed(1)}s`),
    `--glow-color:${rgba(reglages.accent, intensite.glow)}`,
  ].join(';');
}

// ------------------------------------------------------------------
//  Les allers-retours avec Rust
// ------------------------------------------------------------------

/**
 * Premier demarrage de l'interface : les reglages, puis l'etat du poste.
 *
 * L'ordre compte. Les reglages portent le profil memorise, et c'est lui
 * qu'on soumet a `etat` pour qu'il soit recoupe avec les profils qui
 * existent vraiment.
 */
export async function demarrer(): Promise<void> {
  chargerDisposition();
  if (!pont.pontPresent()) {
    // Ouverte dans un navigateur pour une verification de rendu. On
    // affiche l'interface avec ses valeurs par defaut plutot que rien.
    ui.assistant = true;
    ui.pret = true;
    return;
  }
  try {
    // Le contrat d'abord : s'il ne colle pas, tout ce qui suit peut
    // arriver amoindri, et mieux vaut le dire que laisser chercher.
    try {
      const diag = await pont.diagnostic();
      systeme.diagnostic = diag;
      systeme.contratDepareille = diag.contrat !== pont.CONTRAT_ATTENDU;
    } catch {
      systeme.contratDepareille = true;
    }
    Object.assign(reglages, await pont.reglages());
    fixerLangue(reglages.langue);
    chargerDisposition();
    ui.assistant = !reglages.assistant_fait;
    ui.choix = reglages.profil ?? '';
    await releverEtat();
  } catch (e) {
    systeme.erreur = String(e);
  } finally {
    ui.pret = true;
  }
}

// ------------------------------------------------------------------
//  La navigation
// ------------------------------------------------------------------

/**
 * Va sur un ecran.
 *
 * L'ecran change tout de suite, sans attendre Rust : un affichage qui
 * attend un aller-retour pour reagir a un clic se sent, meme quand
 * l'aller-retour est local. Rust rend ensuite l'etat de l'historique,
 * qui est la verite et qui repose l'ecran si jamais les deux avaient
 * diverge.
 */
export async function naviguer(ecran: string): Promise<void> {
  if (ui.ecran === ecran) return;
  ui.ecran = ecran;
  if (!pont.pontPresent()) return;
  try {
    appliquerNavigation(await pont.naviguer(ecran));
  } catch (e) {
    systeme.erreur = String(e);
  }
}

/**
 * Revient a l'ecran precedent, comme le bouton Precedent d'un
 * navigateur.
 *
 * Une surimpression ouverte se ferme d'abord, **sans consommer d'entree
 * d'historique**. C'est ce que tout le monde attend d'un retour quand
 * quelque chose est ouvert par dessus, et une surimpression n'est pas
 * un ecran : elle n'a jamais ete empilee.
 */
export async function reculer(): Promise<void> {
  if (ui.selecteurProfil) {
    ui.selecteurProfil = false;
    return;
  }
  if (!pont.pontPresent()) return;
  try {
    appliquerNavigation(await pont.reculer());
  } catch (e) {
    systeme.erreur = String(e);
  }
}

/** Repart vers l'avant, apres un retour. */
export async function avancer(): Promise<void> {
  if (!pont.pontPresent()) return;
  try {
    appliquerNavigation(await pont.avancer());
  } catch (e) {
    systeme.erreur = String(e);
  }
}

function appliquerNavigation(etat: pont.EtatNavigation): void {
  ui.ecran = etat.ecran;
  ui.peutReculer = etat.peut_reculer;
  ui.peutAvancer = etat.peut_avancer;
}

/**
 * Le coup d'oeil des cinq secondes.
 *
 * Il remplace le releve complet que la boucle faisait jusqu'au lot 19d.
 * Rust ne regarde plus que deux choses, la liste des processus et la
 * date du dossier des profils, et ne rend la main au releve complet que
 * quand ce dossier a bouge. Ce qui change a l'ecran, l'etat du jeu, est
 * deja dans la reponse : aucun aller-retour de plus.
 */
export async function veiller(): Promise<void> {
  const veille = await pont.veille();
  systeme.jeuLance = veille.jeu_lance;
  if (veille.profils_a_relire) await releverEtat();
}

/** Relit ce que le systeme dit de l'installation. */
export async function releverEtat(): Promise<void> {
  const etat = await pont.etat(reglages.profil);
  systeme.cheminJeu = etat.chemin_jeu;
  systeme.jeuLance = etat.jeu_lance;
  systeme.profils = etat.profils;
  // Le profil actif vient de Rust, qui a recoupe le profil memorise avec
  // ceux qui existent. S'il a change, c'est que le memorise n'existe plus,
  // et le nouveau devient le memorise.
  if (etat.profil_actif && etat.profil_actif !== reglages.profil) {
    reglages.profil = etat.profil_actif;
    if (!ui.choix) ui.choix = etat.profil_actif;
    enregistrer();
  }
}

/**
 * Met ou enleve un serveur des favoris.
 *
 * Passe par Rust et pas par la liste locale : la borne sur le nombre de
 * favoris vit dans `cairn-core`, ou elle est testee, et l'etoile doit
 * prendre effet tout de suite alors que l'ecriture des reglages est
 * volontairement differee.
 */
export async function basculerFavori(login: string): Promise<void> {
  if (!pont.pontPresent()) {
    // Hors de la fenetre Tauri, pour les verifications de rendu.
    const i = reglages.favoris.indexOf(login);
    if (i === -1) reglages.favoris.push(login);
    else reglages.favoris.splice(i, 1);
    return;
  }
  try {
    // On ne repose QUE les favoris, et pas tous les reglages que Rust
    // rend. Rust travaille sur sa copie en memoire, qui date du dernier
    // enregistrement, et celui-ci est volontairement differe de 400 ms :
    // etoiler un serveur juste apres avoir replie un groupe remettrait
    // sinon ce groupe comme il etait. Le champ qui vient de changer est
    // le seul qui fasse autorite ici.
    const retenus = await pont.basculerFavori(login);
    reglages.favoris = retenus.favoris;
  } catch (e) {
    systeme.erreur = String(e);
  }
}

export async function chargerServeurs(): Promise<void> {
  try {
    systeme.serveurs = await pont.serveurs();
    systeme.serveursCharges = true;
    systeme.erreur = null;
  } catch (e) {
    systeme.erreur = String(e);
  }
}

// ------------------------------------------------------------------
//  L'enregistrement des reglages
// ------------------------------------------------------------------

let attente: ReturnType<typeof setTimeout> | undefined;

/**
 * Enregistre les reglages, un peu plus tard.
 *
 * Le delai n'est pas du confort : un selecteur de couleur emet un
 * evenement par pixel de deplacement du curseur, et chaque evenement
 * ecrirait un fichier. On attend donc que ca se calme.
 */
export function enregistrer(): void {
  if (!pont.pontPresent()) return;
  clearTimeout(attente);
  attente = setTimeout(() => {
    // $state.snapshot rend un objet ordinaire : un proxy de Svelte ne
    // traverse pas le pont, la serialisation echouerait.
    pont
      .enregistrerReglages($state.snapshot(reglages))
      .then(appliquerRetour)
      .catch((e) => {
        systeme.erreur = String(e);
      });
  }, 400);
}

/**
 * Enregistre tout de suite, et attend que ce soit fait.
 *
 * Pour les changements dont on veut lire l'effet juste apres, comme le
 * chemin du jeu designe a la main : Rust garde les reglages en memoire,
 * il faut donc qu'il les ait recus avant qu'on lui redemande l'etat.
 */
export async function enregistrerMaintenant(): Promise<void> {
  if (!pont.pontPresent()) return;
  clearTimeout(attente);
  appliquerRetour(await pont.enregistrerReglages($state.snapshot(reglages)));
}

/**
 * Repose ce que Rust a retenu, quand ce n'est pas ce qu'on avait propose.
 *
 * Le cas normal est que rien ne change. Le cas qui compte est celui d'un
 * raccourci pose sur une touche reservee : l'interface l'affichait deja,
 * Rust l'a refuse, et c'est ici que l'affichage revient a la verite.
 * Sans ca il faudrait dupliquer la regle de validation en TypeScript.
 */
function appliquerRetour(retenus: pont.Reglages): void {
  for (const [cle, valeur] of Object.entries(retenus)) {
    const courant = (reglages as unknown as Record<string, unknown>)[cle];
    // On ne repose que ce qui differe, pour ne pas reveiller toute
    // l'interface a chaque enregistrement.
    if (JSON.stringify(courant) !== JSON.stringify(valeur)) {
      (reglages as unknown as Record<string, unknown>)[cle] = valeur;
    }
  }
}

/** Change la langue et la retient. */
export function changerLangue(valeur: 'fr' | 'en'): void {
  reglages.langue = valeur;
  fixerLangue(valeur);
  enregistrer();
}

/**
 * Repond a la question posee par la croix.
 *
 * Le choix part a Rust, qui reduit ou ferme et qui retient la reponse
 * si la case etait cochee. L'interface ne decide de rien ici : la
 * fenetre n'est pas a elle.
 */
export function repondreFermeture(choix: 'reduire' | 'quitter', memoriser: boolean): void {
  if (!pont.pontPresent()) return;
  if (memoriser) reglages.fermeture = choix;
  pont.fermerOuReduire(choix, memoriser).catch((e) => {
    systeme.erreur = String(e);
  });
}

/**
 * Pousse une tuile de l'accueil d'un cran.
 *
 * Le chemin du clavier : Ctrl et une fleche sur l'accueil, les deux
 * fleches dans les reglages. Le chemin de la souris est
 * [`glisserTuiles`], et les deux passent par la meme regle en Rust, ce
 * qui garantit qu'ils ne divergent pas. Ici l'ordre est enregistre a
 * chaque poussee : une poussee est un geste fini, contrairement a un
 * glisser qui en enchaine plusieurs avant qu'on lache.
 */
export async function rangerTuiles(
  grille: pont.Grille,
  mouvement: { id: string; pas: number },
): Promise<void> {
  if (!pont.pontPresent()) return;
  try {
    const ordre = ordreDe(grille);
    poserOrdre(grille, await pont.decalerTuile(grille, ordre, mouvement.id, mouvement.pas));
    enregistrer();
  } catch (e) {
    systeme.erreur = String(e);
  }
}

/**
 * L'ordre courant d'une grille, et la façon de le reposer.
 *
 * Deux champs pour deux ecrans, et un seul endroit qui sait lequel est
 * lequel. C'est ce qui evite que chaque appelant ait a se souvenir que
 * celui de Jouer s'appelle `ordre_tuiles` tout court, pour la raison
 * historique ecrite cote Rust.
 */
function ordreDe(grille: pont.Grille): string[] {
  return $state.snapshot(
    grille === 'editer' ? reglages.ordre_tuiles_editer : reglages.ordre_tuiles,
  );
}

function poserOrdre(grille: pont.Grille, ordre: string[]): void {
  if (grille === 'editer') reglages.ordre_tuiles_editer = ordre;
  else reglages.ordre_tuiles = ordre;
}

/**
 * Une tuile portee au dessus d'une autre, pendant un glisser.
 *
 * Appelee a chaque fois que le pointeur entre dans une autre tuile, donc
 * plusieurs fois par geste. Elle ne passe pas par [`rangerTuiles`] pour
 * une seule raison : celle-la enregistre les reglages, et ecrire le
 * fichier a chaque croisement reviendrait a taper sur le disque pendant
 * tout le mouvement. L'ecran appelle `enregistrer()` une fois, au
 * lacher, et seulement si quelque chose a bouge.
 *
 * Rend `true` quand l'ordre a change, pour que l'ecran sache s'il y a
 * quelque chose a garder.
 */
export async function glisserTuiles(
  grille: pont.Grille,
  depuis: string,
  vers: string,
): Promise<boolean> {
  if (!pont.pontPresent()) return false;
  try {
    const ordre = ordreDe(grille);
    const suivant = await pont.glisserTuile(grille, ordre, depuis, vers);
    if (suivant.length === ordre.length && suivant.every((t, i) => t === ordre[i])) {
      return false;
    }
    poserOrdre(grille, suivant);
    return true;
  } catch (e) {
    systeme.erreur = String(e);
    return false;
  }
}

/**
 * Relit l'inventaire des fichiers du joueur.
 *
 * Volontairement **hors de la boucle de veille**. Parcourir un dossier
 * `Downloaded` qui compte des milliers de maps toutes les cinq secondes
 * serait exactement le defaut que le lot 19d a corrige ailleurs : c'est
 * l'ouverture de l'ecran qui declenche la lecture, et un bouton qui la
 * redemande quand le joueur revient du jeu.
 */
export async function chargerInventaire(): Promise<void> {
  if (!pont.pontPresent()) {
    systeme.inventaireCharge = true;
    return;
  }
  systeme.inventaireEnCours = true;
  try {
    systeme.inventaire = await pont.inventaire();
    systeme.inventaireCharge = true;
  } catch (e) {
    systeme.erreur = String(e);
  } finally {
    systeme.inventaireEnCours = false;
  }
}

/** Affiche un message fugace en bas de la fenetre. */
let messageTimer: ReturnType<typeof setTimeout> | undefined;
export function message(texte: string): void {
  ui.message = texte;
  clearTimeout(messageTimer);
  messageTimer = setTimeout(() => {
    ui.message = null;
  }, 2600);
}
