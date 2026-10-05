# Publier les actualités de Cairn

Ce dossier contient le fil d'essai (`actualites.json`), sa signature (`actualites.json.sig`) et ce mode d'emploi.

Le dispositif tient en une phrase : **un fichier JSON public, signé, posé sur un hébergement statique.** Pas de serveur à administrer, pas de compte, pas de base de données, aucune donnée personnelle. Cairn le télécharge, vérifie la signature, le garde en cache et l'affiche.

## Ce qu'il faut faire une seule fois

### 1. Fabriquer le trousseau

Le fichier de clé privée est écrit **dans le dossier d'où tu lances la commande**, pas dans celui de la caisse. Lance-la donc depuis un dossier **en dehors** du dépôt, sinon ta clé privée atterrit dedans :

```powershell
cd "C:\Users\<toi>\Mes Documents"
cargo run --manifest-path "<chemin>\Cairn\crates\cairn-core\Cargo.toml" --example actualites -- cle
```

Ça affiche une clé publique et écrit `cairn-actualites-privee.txt` à côté de toi.

- La **clé publique** se recopie dans `crates/cairn-core/src/lib.rs`, à la place de `CLE_ACTUALITES`.
- La **clé privée** reste chez toi. Pas dans le dépôt, pas dans un ticket, pas dans une conversation, pas dans un message privé. Le dépôt des actualités porte un `.gitignore` qui l'écarte, mais un `.gitignore` ne rattrape pas un fichier déposé à la main dans une autre arborescence. C'est le seul secret du dispositif, et c'est aussi tout ce qu'il demande de garder.

Si elle fuite, n'importe qui peut fabriquer un fil que tous les Cairn installés croiront. Si elle est perdue, rien n'est cassé : tu refais un trousseau, tu remplaces la constante et tu publies une version. Les Cairn déjà installés cessent de recevoir du neuf jusqu'à la mise à jour, et gardent ce qu'ils ont en cache.

La clé qui est dans le dépôt aujourd'hui est une **clé de développement**, sa moitié privée est dans `outils/`, donc connue de tout le monde. Elle ne sert qu'aux essais.

### 2. Choisir l'hébergement

Décidé le 2026-10-05 : une organisation GitHub, `cairn-sm-launcher`, avec deux dépôts publics, `cairn` pour le launcher et `cairn-actualites` pour le fil. Les fichiers prêts à déposer dans le second sont dans `docs/actualites/depot/`.

Le fil est servi par GitHub Pages, donc aux adresses :

```
https://cairn-sm-launcher.github.io/cairn-actualites/actualites.json
https://cairn-sm-launcher.github.io/cairn-actualites/actualites.json.sig
```

Elles se règlent dans `cairn-core`, constantes `URL_ACTUALITES` et `URL_ACTUALITES_SIGNATURE`.

**Le dépôt doit être public.** GitHub Pages n'est disponible que sur les dépôts publics avec une organisation gratuite.

**Compte avec quelques minutes de retard à chaque publication.** Pages passe par un cache, donc un fil qui vient d'être déposé n'arrive pas forcément à la seconde. Le bouton Actualiser de l'écran Communauté redemande le fichier, mais il ne vide pas le cache de GitHub.

Le HTTPS prouve que le fichier vient bien de l'hébergeur. La signature prouve qu'il vient de toi, même si le compte de l'hébergeur est pris. Les deux ne font pas double emploi, c'est pour ça qu'il y a les deux.

## Publier un article

Un article est un objet du tableau `articles` :

```json
{
  "id": "2026-10-04-coupe-atria",
  "titre": "Coupe Atria, samedi 21h",
  "categorie": "tournoi",
  "date": 1791013600,
  "resume": "Inscriptions ouvertes jusqu'a vendredi soir.",
  "langue": "fr",
  "epingle": false,
  "corps": "Le texte, en markdown."
}
```

- **`id`** : unique et stable. La date en tête rend le tri lisible à l'œil.
- **`categorie`** : `tournoi`, `maps`, `creation`, `technique`, `entraide`. Autre chose est rangé dans « Autre » au lieu de faire perdre l'article, donc une catégorie inventée ne casse rien chez ceux qui n'ont pas mis à jour.
- **`date`** : secondes depuis 1970.
- **`langue`** : `fr`, `en`, ou **vide**. Vide veut dire que l'article vaut dans les deux langues. Un article marqué `fr` n'apparaît pas sur une interface anglaise.
- **`epingle`** : passe devant les autres, quelle que soit sa date.
- **`corps`** : le markdown.

Et le fil lui-même porte `version` (toujours `1` pour l'instant) et **`publie`**, l'horodatage de l'assemblage. **`publie` doit augmenter à chaque publication** : Cairn refuse un fil antérieur à celui qu'il a déjà, ce qui empêche quelqu'un qui contrôlerait l'hébergement de reservir un vieux fil correctement signé pour faire disparaître une annonce.

### Le markdown accepté

Titres `#`, `##`, `###`. Paragraphes. Listes `- ` et `1. `. Citations `> `. Blocs de code encadrés par ```` ``` ````. Séparateur `---`. En ligne : `**gras**`, `*italique*`, `` `code` `` et `[texte](https://…)`.

Deux limites à connaître :

- **Les liens ne peuvent être que `http` ou `https`.** Un lien vers un fichier ou un programme est affiché comme du texte, sans destination. Un article est écrit par quelqu'un d'autre et ce lien part dans un appel système : c'est tout ce qu'il faut savoir pour fermer la porte au lieu de faire confiance.
- **Les images deviennent des liens nommés.** La politique de sécurité de Cairn n'autorise aucune image distante, donc une image resterait un cadre vide. Un lien dit au moins ce qu'il y a au bout.

Ce qui ressemble à du HTML reste du texte : Rust découpe le markdown en blocs et l'interface pose des nœuds, il n'y a nulle part de chaîne HTML. Rien à échapper, donc rien à oublier d'échapper.

### Signer et mettre en ligne

```
cargo run --example actualites -- signer cairn-actualites-privee.txt actualites.json
```

Ça écrit `actualites.json.sig`. **Publie les deux fichiers côte à côte**, et dans cet ordre : d'abord le `.json`, puis le `.sig`. L'inverse laisse une fenêtre où la signature ne correspond pas au fil, pendant laquelle Cairn refuse tout et garde son cache.

La signature porte sur les **octets du fichier**, pas sur un JSON relu puis réécrit. Si tu modifies le `.json` après l'avoir signé, ne serait-ce que d'un espace, il faut resigner.

## La modération

Elle n'a pas besoin d'outil : c'est la revue d'une proposition de modification sur le dépôt. Quelqu'un propose un article, tu le lis, tu acceptes ou tu refuses, et l'historique garde qui a écrit quoi et quand. C'est exactement l'exigence du plan, avec l'historique en prime et sans une ligne de code à maintenir.

Ce qui manque à cette version : les mentions j'aime, les commentaires et le partage interne. Ils demandent des comptes, donc un hub, donc le lot 27. Sans eux le launcher reste complet.
