# Cairn

Launcher communautaire pour **ShootMania**. Il ouvre le jeu directement sur le serveur choisi, sans passer par les menus natifs.

**Projet indépendant, non affilié à Nadeo ni à Ubisoft.** ShootMania et ManiaPlanet sont leurs marques.

---

## Ce qu'il fait aujourd'hui

- **Lancer une partie en un clic.** Cairn trouve ManiaPlanet, lit les profils déjà présents sur la machine, liste les serveurs en ligne et ouvre le jeu sur celui qu'on choisit.
- **Parcourir les serveurs** avec des colonnes, des filtres comptés, une recherche et un tirage au sort.
- **Des favoris**, qui lui appartiennent et vivent dans ses propres réglages.
- **Un inventaire** de tes maps, de tes replays et des title packs installés.
- **Un fil d'actualités** de la communauté, signé et vérifié, qui marche aussi hors ligne.
- **Un lecteur de journal** avec export anonymisé, pour qu'un rapport de bogue soit lisible.

Un classement et un historique existent à l'écran, sur des données d'exemple : ils attendent le serveur de match de la phase 4.

## Ce qu'il ne fait pas, et ne fera pas

- **Il ne demande jamais ton mot de passe ManiaPlanet.** Il lance le jeu avec un nom de profil et une adresse de serveur, rien d'autre. Aucun mot de passe ne transite ni n'est stocké.
- **Il n'envoie rien sur toi.** Pas de compte, pas de télémétrie, pas de statistiques d'usage. Les seules sorties réseau sont la liste publique des serveurs ManiaPlanet et le fil d'actualités.
- **Il ne modifie pas le jeu.** Ce n'est pas un mod et il ne s'accroche à aucun processus.

## Construire

Il faut [Rust](https://rustup.rs/) 1.77 ou plus, [Node](https://nodejs.org/) 20 ou plus, et les [prérequis Tauri](https://tauri.app/start/prerequisites/) pour Windows.

```
npm install
npm run tauri dev
```

Pour un exécutable :

```
npm run tauri build
```

## Comment c'est fait

| Dossier | Ce qu'il y a dedans |
|---|---|
| `crates/cairn-core` | Le cœur métier, **sans entrée-sortie, sans Tauri, sans code propre à un système**. Tout s'y teste en une seconde. |
| `src-tauri` | La plomberie : registre, processus, fichiers, réseau, fenêtre. |
| `src` | L'interface, en Svelte 5 et TypeScript. |
| `verification` | Un banc d'essai qui pilote l'application avec un vrai navigateur. |

Deux règles tiennent l'architecture, et elles expliquent presque tout le découpage :

1. **Ce qui mérite un test vit dans `cairn-core`.** Les lignes de commande de lancement, l'analyse du texte Trackmania, le barème de rang, l'anonymisation du journal, la vérification de signature. `src-tauri` appelle le système et poste le résultat ici.
2. **Tous les appels vers Rust passent par `src/lib/pont.ts`**, et nulle part ailleurs. Un numéro de contrat compare les deux côtés au démarrage, parce qu'une interface rechargée à chaud contre un binaire périmé est une soirée perdue.

## Contribuer

Les propositions de modification sont bienvenues. Avant d'en ouvrir une :

```
cargo test --manifest-path crates/cairn-core/Cargo.toml
cargo clippy --manifest-path crates/cairn-core/Cargo.toml --all-targets -- -D warnings
npm run check
```

Le projet est écrit en français, commentaires compris. Les messages affichés sont en anglais dans le code et traduits par `src/lib/fr.ts`, les clés étant les phrases anglaises.

Pour proposer une **actualité** et non du code, c'est dans [cairn-actualites](https://github.com/cairn-sm-launcher/cairn-actualites).

## Sécurité

Si tu trouves une faille, ouvre un ticket en restant vague sur les détails, ou contacte directement le mainteneur. Les sujets sensibles : ce qui part dans une ligne de commande, ce qui ouvre un lien, et la vérification du fil d'actualités.

Le fil est signé avec une clé Ed25519 dont la moitié publique est compilée dans le launcher. La clé présente dans `outils/` est une clé d'essai sans valeur, voir `outils/LISEZMOI.md`.

## Licence

[MIT](LICENSE).
