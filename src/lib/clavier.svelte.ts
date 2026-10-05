// L'etiquette reelle des touches.
//
// Les raccourcis sont retenus par la **position** de la touche, jamais
// par sa lettre : `KeyQ` est la meme touche physique partout, etiquetee
// A sur un clavier francais et Q sur un clavier anglais. Reste a
// afficher la bonne lettre sur la pastille, et c'est le systeme qui la
// connait, pas nous.
//
// `navigator.keyboard.getLayoutMap()` rend cette correspondance. Elle
// n'existe pas partout, d'ou le repli de `cairn-core::touches`, qui
// suppose un QWERTY. Un repli faux chez Flow, et assume : mieux vaut
// « Q » sur une pastille que rien du tout, et la fenetre de Cairn est un
// WebView2, donc du Chromium, qui sait repondre.
//
// L'interet par rapport a une table par langue : il n'y a aucune liste
// de dispositions a tenir a jour, et un joueur qui change de clavier
// voit la pastille changer toute seule.

import { repliEtiquette } from './touchesRepli';

type ClavierApi = {
  keyboard?: { getLayoutMap?: () => Promise<Map<string, string>> };
};

/** La correspondance position vers etiquette, vide tant qu'on ne l'a pas. */
let carte = $state<Map<string, string> | null>(null);
let demandee = false;

/**
 * Demande la disposition au systeme, une seule fois.
 *
 * Silencieuse en cas d'echec : l'absence de cette interface n'est pas
 * une panne, c'est un navigateur qui ne la propose pas, et le repli
 * couvre le cas.
 */
export function chargerDisposition(): void {
  if (demandee || typeof navigator === 'undefined') return;
  demandee = true;
  const api = (navigator as Navigator & ClavierApi).keyboard;
  api
    ?.getLayoutMap?.()
    .then((m) => {
      carte = m;
    })
    .catch(() => {
      // Le repli fera l'affaire.
    });
}

/**
 * L'etiquette a montrer pour une position de touche.
 *
 * Rend une majuscule : une pastille de raccourci porte « A », pas « a ».
 */
export function etiquette(code: string): string {
  const reelle = carte?.get(code);
  const brute = reelle && reelle.length > 0 ? reelle : repliEtiquette(code);
  return brute.length === 1 ? brute.toUpperCase() : brute;
}
