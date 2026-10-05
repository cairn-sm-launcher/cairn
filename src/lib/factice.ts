/**
 * Des données inventées, le temps de construire les écrans.
 *
 * **Ce fichier est jetable, et il doit le rester.** Il disparaît à
 * l'étape 35, quand le hub rendra les vraies données. Tout ce qui le
 * consomme doit donc passer par les types de `pont.ts`, jamais par des
 * formes définies ici : le jour où on le supprime, seule la source
 * change, pas les écrans.
 *
 * La même méthode qu'au lot 19c : la liste des serveurs a été construite
 * sur un échantillon avant que l'API réponde, et le branchement n'a
 * ensuite rien coûté.
 *
 * **Pourquoi des données réalistes plutôt que « Joueur 1, Joueur 2 ».**
 * Une mise en page se juge sur ce qui la déforme. Des pseudos courts et
 * réguliers font croire que tout va bien, puis le premier vrai pseudo à
 * rallonge casse la colonne. Les noms ci-dessous sont donc de vraies
 * longueurs, avec des accents, et les maps sont celles du dossier de
 * Flow.
 */

/** Une ligne du classement. */
export type LigneClassement = {
  place: number;
  joueur: string;
  /** L'indice du palier, de 0 à 4. */
  palier: number;
  points: number;
  victoires: number;
  defaites: number;
  /** Vrai pour la ligne du joueur qui regarde. */
  soi: boolean;
};

/** Une partie jouée. */
export type Partie = {
  id: string;
  gagne: boolean;
  /** Les points de rang gagnés ou perdus. */
  delta: number;
  score: number;
  scoreAdverse: number;
  /** Captures, lasers, roquettes, pour le joueur. */
  moi: [number, number, number];
  /** Les mêmes, pour son équipe entière. */
  equipe: [number, number, number];
  map: string;
  /** Secondes depuis 1970. */
  date: number;
};

const JOUEURS: Array<[string, number, number, number, number]> = [
  // pseudo, palier, points, victoires, défaites
  ['Δ Riolu', 4, 62, 148, 71],
  ['nocturne', 4, 41, 131, 78],
  ['Kʁaken', 4, 18, 119, 74],
  ['pti_suisse', 3, 88, 96, 62],
  ['MarbleHornets', 3, 71, 88, 60],
  ['Xirdrak', 3, 34, 74, 58],
  ['Ludo.', 3, 12, 69, 55],
  ['Tenshi', 2, 90, 61, 52],
  ['bouloche', 2, 64, 57, 51],
  ['Sanglier des Landes', 2, 47, 54, 49],
  ['zk', 2, 21, 48, 46],
  ['Príncipe', 1, 83, 41, 44],
  ['Mo0n', 1, 55, 36, 41],
  ['Ada', 1, 29, 31, 38],
  ['petitpois', 0, 76, 22, 31],
  ['nouveau_venu', 0, 14, 6, 9],
];

/** Le classement, du meilleur au moins bon. */
export const CLASSEMENT: LigneClassement[] = JOUEURS.map(
  ([joueur, palier, points, victoires, defaites], i) => ({
    place: i + 1,
    joueur,
    palier,
    points,
    victoires,
    defaites,
    soi: joueur === 'Xirdrak',
  }),
);

/** La ligne du joueur qui regarde, pour la tête de l'écran. */
export const MOI: LigneClassement =
  CLASSEMENT.find((l) => l.soi) ?? CLASSEMENT[CLASSEMENT.length - 1];

const MAPS = [
  'Chinatown',
  'PIXEL - Chinatown',
  'BXS',
  'RedRock',
  'Autumn',
  'Frostline',
  'Start',
];

/**
 * Les parties récentes.
 *
 * **Les victoires sont écrites, pas tirées.** Une suite pseudo-aléatoire
 * réglée pour donner 60 % de victoires en a donné 46 %, et le joueur
 * classé sixième s'est retrouvé avec un bilan négatif dans son propre
 * historique. Corriger en cherchant une meilleure graine aurait tenu
 * jusqu'à la prochaine retouche du générateur : la moindre ligne ajoutée
 * décale toute la suite. Le motif ci-dessous vaut 14 victoires sur 24,
 * soit 58 %, ce qui correspond au 74 / 58 de son classement, et il le
 * vaudra toujours. Le hasard ne sert plus qu'aux chiffres de détail, où
 * il n'engage rien.
 *
 * Fabriquées par une suite déterministe et non par un tirage au sort :
 * une page qui change de contenu à chaque rechargement est impossible à
 * comparer d'une capture à l'autre, et le banc d'essai ne pourrait rien
 * en dire.
 */
/** Le déroulé des vingt-quatre parties, de la plus récente à la plus ancienne. */
const RESULTATS = 'VVDVVVDVDVVDDVVDVVDVDVDD';

export const PARTIES: Partie[] = (() => {
  const sortie: Partie[] = [];
  // Un jour de référence fixe, pour que les captures soient comparables.
  const DEPART = Date.UTC(2026, 8, 29) / 1000;
  let graine = 29;
  /**
   * Le bit de poids faible d'un générateur congruentiel alterne
   * simplement, donc `graine % 2` donne 0, 1, 0, 1 et `graine % 10` n'est
   * guère mieux. On lit les bits hauts, qui eux se mélangent.
   */
  const suivant = (borne: number) => {
    graine = (graine * 1103515245 + 12345) % 2147483648;
    return Math.floor((graine / 65536) % borne);
  };
  for (let i = 0; i < 24; i++) {
    const gagne = RESULTATS[i] === 'V';
    const score = gagne ? 9 : 4 + suivant(5);
    const scoreAdverse = gagne ? suivant(9) : 9;
    const moi: [number, number, number] = [suivant(4), 2 + suivant(9), 4 + suivant(12)];
    sortie.push({
      id: `p${i}`,
      gagne,
      delta: gagne ? 14 + suivant(16) : -(12 + suivant(14)),
      score,
      scoreAdverse,
      moi,
      // Le total de l'équipe **contient** celui du joueur, il n'est pas
      // tiré à côté. Tirés séparément, les deux se croisaient : un joueur
      // à dix lasers dans une équipe qui en comptait huit. C'est le même
      // défaut que les dates dans le désordre, invisible au premier coup
      // d'œil et faux dès qu'on lit la ligne.
      equipe: [
        moi[0] + 1 + suivant(4),
        moi[1] + 5 + suivant(11),
        moi[2] + 7 + suivant(16),
      ] as [number, number, number],
      map: MAPS[suivant(MAPS.length)],
      date: 0, // pose juste apres, voir plus bas
    });
  }
  // Les dates sont posees apres coup, en descendant.
  //
  // La premiere version les tirait dans la boucle avec un ecart au
  // hasard, et l'ordre s'en trouvait rompu : Sep 25 suivi de Sep 23 puis
  // de Sep 24. Un historique qui remonte le temps par endroits est faux,
  // et c'est le genre de faux qu'on ne voit qu'en lisant la colonne de
  // haut en bas, donc autant le rendre impossible.
  let jour = DEPART;
  for (const partie of sortie) {
    partie.date = jour;
    jour -= 86400 * (1 + suivant(3));
  }
  return sortie;
})();

