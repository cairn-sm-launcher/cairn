// La traduction.
//
// La maquette rendait la page en anglais puis reecrivait la chaine HTML
// avec une expression reguliere, juste avant de l'injecter dans le DOM.
// Le procede tenait tant qu'il y avait une chaine a reecrire. Svelte
// construit des noeuds, il n'y a plus de chaine, donc la traduction
// redevient ce qu'elle aurait toujours du etre : un appel de fonction.
//
// Le dictionnaire, lui, n'a pas bouge d'une ligne. Les cles restent les
// phrases anglaises, voir fr.ts.

import { FR } from './fr';

export type Langue = 'fr' | 'en';

// Les tournures qui portent une valeur, que le dictionnaire ne peut pas
// couvrir puisqu'elles changent a chaque affichage. Elles etaient des
// expressions regulieres appliquees au HTML fini, avec les faux positifs
// que ca suppose : le motif « On » devait etre ecrit « >On ([A-Z]) » pour
// ne pas attraper le mot « on » d'une phrase. Ce sont maintenant des
// modeles a trous, donc le probleme n'existe plus.
const MOTIFS: Record<string, string> = {
  '{0}/{1} players': '{0}/{1} joueurs',
  '{0} online': '{0} en ligne',
  'On {0}': 'Sur {0}',
  'Theme {0}': 'Thème {0}',
  'Joining {0}...': 'Connexion à {0}...',
  'Spectating {0}...': 'Observation de {0}...',
  'Last used {0}': 'Vu le {0}',
  '{0} servers, sorted by players': '{0} serveurs, du plus peuplé au moins peuplé',
  'by {0}': 'par {0}',
  'Setup, step {0} of 3': 'Installation, étape {0} sur 3',
  'Slide {0}': 'Vue {0}',
  'Relay of {0}': 'Relais de {0}',
  '{0} shown': '{0} affichés',
  'At random among {0}': 'Au hasard parmi {0}',
  '{0} with players': '{0} avec du monde',
  '{0} empty': '{0} vides',
  '{0} watching': '{0} en observation',
  'Sort by {0}': 'Trier par {0}',
  '+{0} more': '+{0} autres',
  '{0} servers saved': '{0} serveurs enregistrés',
  '{0} ranked players': '{0} joueurs classés',
  '{0} W / {1} L': '{0} V / {1} D',
  '{0} points behind {1}': '{0} points derrière {1}',
  '{0} wins, {1} losses': '{0} victoires, {1} défaites',
  '{0} older lines are not shown.': '{0} lignes plus anciennes ne sont pas affichées.',
  'Report saved as {0}, in the log folder.': 'Rapport enregistré sous {0}, dans le dossier du journal.',
  'Cairn {0}, engine contract {1}, on {2}.': 'Cairn {0}, contrat moteur {1}, sur {2}.',
  'Updated {0}': 'Mis à jour le {0}',
  '{0} Showing the last ones kept on this computer.':
    '{0} Voici les dernières gardées sur cet ordinateur.',
};

// La langue est un etat reactif, d'ou l'extension .svelte.ts. Sans ca,
// changer de langue dans les reglages ne redessinerait rien : Svelte ne
// sait pas qu'une variable de module a bouge, alors qu'il suit un $state
// jusque dans les composants qui appellent t().
let langue = $state<Langue>('en');

/** Fixe la langue. Appele au demarrage puis a chaque changement. */
export function fixerLangue(valeur: Langue): void {
  langue = valeur;
}

export function langueCourante(): Langue {
  return langue;
}

/**
 * Traduit une phrase, en remplacant `{0}`, `{1}` par les valeurs donnees.
 *
 * Une phrase absente du dictionnaire est rendue telle quelle, donc en
 * anglais. C'est voulu : un oubli se voit sans casser l'affichage, et il
 * se corrige en ajoutant une ligne a fr.ts.
 */
export function t(phrase: string, ...valeurs: Array<string | number>): string {
  const table = valeurs.length > 0 ? MOTIFS : FR;
  const modele = (langue === 'fr' && table[phrase]) || phrase;
  if (valeurs.length === 0) return modele;
  return modele.replace(/\{(\d+)\}/g, (entier, index: string) => {
    const valeur = valeurs[Number(index)];
    return valeur === undefined ? entier : String(valeur);
  });
}

/**
 * La note sous un nom de profil.
 *
 * `cairn-core` rend une date nue et pas une phrase, pour ne pas imposer
 * sa langue a l'interface. La phrase se compose donc ici.
 */
export function noteProfil(derniereUtilisation: string | null): string {
  if (!derniereUtilisation) return '';
  return t('Last used {0}', derniereUtilisation);
}
