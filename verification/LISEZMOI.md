# Les scripts de vérification

Trois scripts qui contrôlent ce que `cargo test` ne peut pas voir : le
rendu, les mesures de mise en page, et les trous du dictionnaire.

## `verifier-traduction.mjs`

Relève toutes les phrases passées à `t()` dans `src/` et dit lesquelles
manquent à `src/lib/fr.ts`. Sort en erreur s'il en trouve.

```
node verification/verifier-traduction.mjs
```

Aucune dépendance. À lancer avant chaque livraison.

**Sa limite, à connaître** : il ne voit que les chaînes littérales. Une
étiquette passée par variable, `t(etiquette)`, lui échappe. C'est pour
ça que les étiquettes de la fiche serveur sont traduites à la
construction du tableau et pas à l'affichage.

## `verifier-19c.mjs`

Ouvre l'interface construite dans un navigateur, avec un faux pont qui
rejoue ce que Rust rend sur le poste de Flow, et vérifie :

- les mesures du lot 17 à 1100, 1440 et 1920 : hauteur du bandeau,
  position de la première tuile, écart des onglets au centre de la
  fenêtre, repli de l'état du jeu en fenêtre étroite ;
- **les trois densités**, depuis le lot 19d. Ce qui compte ici est que
  `compact` rende exactement la mise en page d'avant : première tuile à
  102 px du haut, 28 px entre les tuiles et les actualités. Les deux
  autres réglages montent, 126 et 66 pour `equilibre`, 156 et 105 pour
  `aere`. Le script signale `compact` s'il a bougé.

  **La gouttière entre tuiles suit la densité** depuis le lot 19d bis, et
  le contrôle de la colonne tourne donc en `compact` pour que ses
  chiffres restent comparables d'un lot à l'autre ;
- la hauteur figée de la carte de l'assistant à ses trois étapes ;
- **l'historique de navigation**, depuis le lot 19e : Précédent remonte
  le chemin réellement parcouru et non la rangée d'onglets, Suivant
  repart, Alt et les flèches font la même chose, les deux touches
  empilent elles aussi, un sous-onglet de Réglages n'empile pas, et au
  bout de la pile la flèche du bandeau s'éteint au lieu de disparaître ;
- **ranger les tuiles**, par les deux chemins. Au clavier : Ctrl et
  flèche déplace et garde le focus sur la tuile déplacée, et le bord ne
  boucle pas. À la souris : un maintien d'un quart de seconde soulève la
  tuile sans bouger d'un pixel, cinq pixels de déplacement la soulèvent
  tout de suite, la case devient creuse, l'étiquette suit le curseur, la
  grille se réorganise pendant le geste, et un relâchement sur place ne
  change rien.

  **Le garde-fou est le clic franc.** La saisie s'installe sur le même
  bouton de souris que l'ouverture d'une tuile, donc c'est la première
  chose qu'un glisser mal borné casserait. Le script clique, et vérifie
  que la tuile s'ouvre encore.

  **Depuis le lot 22c, Playwright exerce vraiment le geste.** Le glisser
  est passé aux événements de pointeur, que Playwright sait produire
  avec un vrai pointeur. La version d'avant fabriquait des `DragEvent`
  à la main, faute de pouvoir déclencher le glisser natif de HTML5 :
  ça vérifiait nos gestionnaires sans jamais toucher la couche de
  glisser du navigateur, et c'est précisément ce qu'elle ne voyait pas
  qui a cassé chez Flow ;
- **l'accord des deux numéros de contrat**, lu dans les sources et non
  dans l'application. Aucun banc d'essai ne peut voir ce désaccord : le
  faux pont rend le numéro qu'on lui donne, donc un moteur resté en
  arrière passerait tous les contrôles et poserait une bannière ambre en
  permanence chez l'utilisateur. Le script compare donc
  `cairn_core::CONTRAT` et `CONTRAT_ATTENDU` dans les fichiers eux-mêmes.
  Il a été écrit le jour où le numéro a été monté d'un côté et pas de
  l'autre ;
- **l'écran Éditer** : les trois légendes, la fiche dépliée et son
  regroupement, une seule fiche ouverte à la fois, le dossier demandé à
  l'ouverture, le titre demandé au jeu, et le fait qu'un title pack au
  nom bricolé **n'ait pas** de bouton d'ouverture. Ce dernier point est
  le contrôle qui compte : ce bouton part dans une ligne de commande.
  Les tuiles d'Éditer se rangent aussi, par le même composant, et le
  script vérifie que les deux ordres sont enregistrés séparément ;
- **la croix de la fenêtre** : le faux pont sait parler l'API
  d'événements de Tauri, donc le script déclenche la question comme la
  vraie croix le ferait, puis vérifie la case décochée, le focus sur
  Réduire et non sur Fermer, Échap qui annule sans rien envoyer, et les
  deux réponses avec et sans mémorisation ;
- **le numéro de contrat du pont** : en accord aucun message, en
  désaccord le message ambre, et absent le message aussi. Ce dernier cas
  est le cas réel, celui d'un binaire Rust plus ancien que l'interface ;
- l'absence d'erreur en console.

**Le faux pont vit dans un littéral gabarit.** Les trois scripts
refusent désormais de démarrer si sa chaîne ne se termine pas comme elle
devrait, ce qui attrape le cas ci-dessous avant la première mesure.
 Un accent grave dans un
commentaire le ferme en plein milieu, `__TAURI_INTERNALS__` n'est jamais
défini, l'application part sur l'assistant de premier lancement et tous
les contrôles mesurent le vide. `node --check` ne voit rien. Les trois
scripts vérifient donc à l'ouverture de chaque page que le faux pont
s'est installé, et s'arrêtent avec la phrase qui dit où chercher.

**Les boutons 3 et 4 de la souris ne se simulent pas** avec
`page.mouse`, qui ne connaît que gauche, milieu et droite. Le script
envoie donc l'événement que le navigateur enverrait. Ça vérifie le
gestionnaire, pas le pilote de souris : le vrai clic se teste à la main.

**L'écran courant se lit sur `data-ecran`**, posé sur la racine du
document. Le deviner à partir de l'onglet allumé ne marche pas, Réglages
et Parcourir les serveurs n'en allument aucun.

Il dépose ses captures dans `verification/captures/`, ou dans le dossier
que la variable `SORTIE` désigne.

## `verifier-serveurs.mjs`

Ouvre l'écran Parcourir les serveurs sur un échantillon qui ressemble à
la vraie liste : quelques serveurs peuplés, une majorité de déserts, un
privé, un lobby, un relais, des noms longs et accentués. Il contrôle :

- le groupement en « avec du monde » et « vides », le second replié ;
- l'atténuation des lignes vides une fois dépliées ;
- les comptes sur chaque puce de mode, avec et sans recherche ;
- la recherche insensible aux accents, `atria` trouvant `Λtría` ;
- les trois tris ;
- le tirage au sort : quatre tirages distincts, une seule ligne ouverte à
  la fois, et le dépliage du groupe qui porte le serveur tiré ;
- l'état « aucun résultat » et son bouton pour effacer les filtres ;
- **le repli des puces de mode**, six visibles et le reste derrière
  « +N autres ». L'échantillon porte exprès plus de six modes ;
- **le sens du tri et son état neutre** : deux clics sur le même critère
  doivent rendre exactement la liste inverse, et un troisième doit rendre
  exactement la liste de départ ;
- **le filtre mot de passe** dans les deux sens ;
- **les favoris de bout en bout** : poser l'étoile, voir l'atténuation
  tomber sur cette ligne et rester sur ses voisines, filtrer sur eux
  seuls, lire le compte sur la puce, puis retirer l'étoile ;
- **l'alignement de la colonne d'état**, mesuré en abscisses. Une seule
  valeur distincte pour toutes les lignes, sinon ce n'est plus une
  colonne ;
- **les couleurs des noms**, depuis le lot 20 : une couleur par morceau,
  le gras, l'italique, l'infobulle qui porte le nom nu, et **qu'une
  rangée colorée fasse exactement la même hauteur qu'une rangée nue**.
  Le faux pont ne rejoue ni l'analyseur de codes ni le calcul de
  contraste, il fournit un découpage tout fait : ce qu'on vérifie ici
  c'est que l'écran le dessine. La règle, elle, est couverte par
  `cargo test` sur les 4096 couleurs que `$RGB` peut écrire ;
- **qu'un nom s'affiche toujours**, même sans découpage. L'échantillon
  porte exprès un serveur sans `nom_segments`. Ce contrôle est né d'un
  vrai défaut, et d'un défaut silencieux : un binaire Rust d'avant le
  lot 20 avec l'interface d'après, `{#each}` sur un champ absent qui ne
  rend rien sans se plaindre, et toute la colonne des noms vide. Il
  restera même quand la cause d'origine aura disparu, parce que c'est la
  classe de défaut qu'il attrape ;
- **l'interrupteur des couleurs du jeu**, qui doit passer des morceaux
  colorés au nom nu sans perdre une lettre ;
- **la hauteur des rangées à sept largeurs de fenêtre.** Ce dernier point
  est né d'un vrai défaut : une règle `@media` mal fermée avait laissé la
  colonne région visible alors que la grille était passée à cinq
  colonnes, et les deux enfants en trop repartaient sur une seconde
  rangée. La hauteur passait de 37 à 66 px, et c'est ce chiffre qui le
  dit d'un coup d'œil.

**Son faux pont rejoue la logique de filtrage en JavaScript**, ce qui est
une duplication assumée et limitée au banc d'essai : ce qu'on vérifie
ici, c'est que l'écran dessine correctement une vue, pas que la vue est
juste. La vue, elle, est vérifiée par `cargo test`.

**Un piège de faux pont, corrigé au lot 19d.** `enregistrer_reglages`
rend côté Rust les réglages **assainis**, donc un objet. Un faux pont qui
rend `undefined` fait échouer `appliquerRetour` et pose une bannière
d'erreur sur toutes les captures, sans que rien ne soit cassé dans
l'application. Les trois faux ponts rendent maintenant un objet, et
celui de `verifier-serveurs.mjs` garde ce que l'interface a écrit.

**Et un piège de script.** Un `.click()` aveugle sur un en-tête de groupe
referme celui qu'une étape précédente avait ouvert, et l'étape suivante
échoue pour une mauvaise raison. D'où `deplierTout()`, qui ne touche
qu'aux groupes encore repliés.

## `verifier-reglages.mjs`

Ouvre l'écran Réglages et contrôle :

- les deux sous-onglets et leurs sections ;
- **la capture d'un raccourci**, de bout en bout : une touche libre est
  retenue, une touche réservée est refusée avec son message et **le
  raccourci précédent est conservé** plutôt que remis par défaut, la
  touche déjà prise par l'autre sens est refusée, Échap annule ;
- **qu'une capture ne navigue pas.** Le raccourci « onglet suivant » est
  une touche comme une autre : appuyer dessus pendant une capture ne doit
  pas changer d'écran au moment même où on essaie de l'affecter ;
- la réorganisation des tuiles et les flèches désactivées aux extrémités ;
- les emplacements de thèmes : enregistrer, changer d'avis, recharger, et
  une remise à zéro qui garde les emplacements ;
- l'interrupteur des animations, qui pose bien `no-anim` sur la racine ;
- **la densité** : la valeur choisie est retenue dans le fichier et
  arrive bien en attribut sur la racine du document ;
- **l'aperçu de la densité** : il suit le **survol** et pas le choix, la
  densité retenue ne bouge pas tant qu'on n'a pas cliqué, et rien ne
  déborde de la boîte aux trois réglages. Ce dernier point compte :
  `aere` est le cas le plus haut, et un aperçu tronqué mentirait sur le
  réglage le plus généreux.

**Attention aux touches simulées.** Les raccourcis portent sur la
**position** de la touche, pas sur la lettre. Un script qui écrit
`keyboard.press('a')` envoie `KeyA`, qui n'est affecté à rien : il faut
`press('q')`, la position que Playwright simule en QWERTY et qui porte A
sur un clavier français.

## `comparer-19c.mjs`

Rend la maquette d'origine, `shootmania-launcher-demo.html`, dans les
mêmes conditions, pour comparer le portage à ce qu'il porte.

## Dépendance

Tous sauf le premier demandent Playwright, qui n'est **pas** dans
`package.json` : il pèse lourd et il télécharge des navigateurs à
l'installation, ce qui n'a rien à faire dans l'installation ordinaire du
projet.

```
npm i --no-save playwright && npx playwright install chromium
node verification/verifier-19c.mjs
node verification/comparer-19c.mjs
```

`comparer-19c.mjs` prend en premier argument le chemin de la maquette, et
cherche par défaut `../ShootMania Launcher POC/shootmania-launcher-demo.html`
à côté du dossier du projet.

La variable `CHROMIUM` force un exécutable précis, ce qui ne sert que
dans un conteneur où Playwright ne gère pas ses propres navigateurs.
