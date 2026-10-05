# Cairn

Un launcher pour ShootMania qui ouvre le jeu directement sur le serveur choisi.

Projet indépendant, sans aucun lien avec Nadeo ni avec Ubisoft, à qui appartiennent les marques ShootMania et ManiaPlanet.

Version 0.1.0. C'est une alpha, elle est testée par deux personnes.

---

## Pourquoi

Rejoindre un serveur dans ManiaPlanet demande de lancer le jeu, d'attendre le menu, d'ouvrir la liste des serveurs, de la trier et d'y retrouver le bon. Tous les soirs, pour les quelques mêmes serveurs.

Rien n'oblige à passer par là. ManiaPlanet accepte une adresse de serveur sur sa ligne de commande, et le protocole `maniaplanet://` sait rejoindre une partie quand le jeu tourne déjà. Cairn se contente d'utiliser ce qui existe.

---

## Que fait Cairn?

Cairn trouve ManiaPlanet tout seul, en lisant le protocole déjà enregistré sur la machine, et va lire les profils que le jeu garde en local. Aucun mot de passe n'entre en jeu.

On affiche les serveurs en ligne avec leurs colonnes, des filtres comptés et une recherche. On peut en garder en favori, ou en tirer un au hasard quand on ne sait pas où aller.

Le launcher résume ce qu'il y a sur le disque, les maps, les replays et les title packs installés, avec les dates et la place occupée. Il n'ouvre aucun fichier `.Gbx` pour autant, il se contente de ce que le système dit d'un dossier.

Cairn affiche un fil d'actualités de la communauté, qui vient d'un fichier public signé. Il le garde en cache, donc il s'affiche même sans réseau.

Il montre son propre journal, avec un bouton qui en fabrique un rapport dont les chemins et les adresses ont été retirés. C'est ce qui transforme un « ça marche pas » en quelque chose d'exploitable.

Un classement et un historique existent à l'écran, mais sur des données inventées. Ils attendent le serveur de match, qui fait partie des travaux en cours.

---

## Ce que Cairn ne fera pas

Cairn ne demandera jamais le mot de passe ManiaPlanet. Il lance le jeu avec un nom de profil et une adresse, rien d'autre. Un launcher communautaire qui réclame les identifiants du jeu est un launcher qu'il ne faut pas installer et nous ne voulons pas avoir à traiter les données sensibles des joueurs.

Notre launcher n'envoie rien sur qui l'utilise. Il n'y a pas de compte à créer et aucun compteur d'usage. Les seules sorties réseau sont la liste publique des serveurs ManiaPlanet et le fichier d'actualités, et elles s'écrivent dans le journal, que l'application montre elle même.

Il ne touche pas au jeu. Ce n'est pas un mod, il ne s'accroche à aucun processus et il ne modifie aucun fichier de ManiaPlanet.

---

## Construire

Il faut [Rust](https://rustup.rs/) 1.77 ou plus récent, [Node](https://nodejs.org/) 20 ou plus récent, et les [prérequis Tauri pour Windows](https://tauri.app/start/prerequisites/).

```
npm install
npm run tauri dev
```

Pour fabriquer un exécutable installable :

```
npm run tauri build
```

Windows seulement pour l'instant. Tout ce qui dépend du système passe par une interface unique, donc un portage Linux serait un module à écrire plutôt qu'une réécriture, mais personne ne l'a demandé.

---

## Comment le code est rangé

`crates/cairn-core` contient tout ce qui se calcule sans toucher au système. Les deux lignes de commande de lancement, l'analyse du texte coloré de Trackmania, le barème de rang, le découpage du markdown des actualités, l'anonymisation du journal, la vérification des signatures. Rien là-dedans n'ouvre un fichier, n'appelle Tauri ni ne connaît Windows. Tout s'y teste en une seconde, sans fenêtre et sans jeu installé. Il y a trois cents tests.

`src-tauri` fait ce que `cairn-core` s'interdit. Lire le registre, chercher le processus du jeu, écrire les réglages, appeler le réseau, tenir la fenêtre.

`src` est l'interface, écrite en Svelte 5 et TypeScript.

`verification` contient un banc d'essai qui pilote l'application dans un vrai navigateur, mesure ce qu'il voit et le relève.

Deux règles expliquent ce découpage, et elles ont été posées dès le départ pour qu'on puisse dire si elles tiennent et c'est le cas.

La première est que toute fonction qui mérite un test vit dans `cairn-core`. Un bug dans un barème de points ou dans une validation de nom de serveur doit pouvoir se reproduire en trois lignes, pas en lançant l'application.

La seconde est que tous les appels vers Rust passent par `src/lib/pont.ts` et nulle part ailleurs. Un numéro de contrat y est inscrit, comparé au démarrage avec celui du moteur. Il existe parce qu'une interface rechargée à chaud contre un binaire périmé a coûté une soirée entière à comprendre.

---

## Contribuer

Les propositions de modification sont bienvenues. Avant d'en ouvrir une :

```
cargo test --manifest-path crates/cairn-core/Cargo.toml
cargo clippy --manifest-path crates/cairn-core/Cargo.toml --all-targets -- -D warnings
npm run check
```

Le code et les commentaires sont en français. Les textes affichés sont écrits en anglais dans les composants et traduits par `src/lib/fr.ts`, où les clés sont les phrases anglaises elles-mêmes. Une phrase oubliée s'affiche donc en anglais lisible plutôt qu'en identifiant technique.

Pour proposer une actualité plutôt que du code, c'est dans [cairn-actualites](https://github.com/cairn-sm-launcher/cairn-actualites).

---

## Sécurité

Pour signaler une faille, ouvrez un ticket ou contactez directement.

Les endroits sensibles sont ceux où une donnée venue de l'extérieur finit dans un appel système. Le nom de profil et l'adresse de serveur partent dans une ligne de commande, et ils sont validés en Rust avant. Les liens d'un article d'actualité partent au navigateur, et seuls `http` et `https` passent. Le fil d'actualités est signé en Ed25519 et refusé si la signature ne correspond pas à la clé compilée dans le launcher.

La clé présente dans `outils/` sert aux essais et sa moitié privée est dans le dépôt. Elle ne protège rien et c'est voulu, voir `outils/LISEZMOI.md`.

---

## Licence

[MIT](LICENSE).
