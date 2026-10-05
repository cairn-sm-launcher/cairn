# Actualités de Cairn

Le fil d'actualités affiché dans [Cairn](https://github.com/cairn-sm-launcher/cairn), le launcher communautaire pour ShootMania.

**Ce dépôt n'est affilié ni à Nadeo ni à Ubisoft.** ShootMania et ManiaPlanet sont leurs marques. Cairn est un projet indépendant, fait par des joueurs.

## Ce qu'il y a ici

Deux fichiers, et c'est tout le dispositif :

- `actualites.json`, le fil
- `actualites.json.sig`, sa signature

Cairn les télécharge, **vérifie la signature** et n'affiche le fil que si elle correspond à la clé embarquée dans le launcher. Un fichier modifié par quelqu'un d'autre, même servi depuis cette adresse, est refusé.

Pas de serveur, pas de compte, pas de base de données, aucune donnée personnelle n'est collectée.

## Proposer une actualité

Ouvre une proposition de modification sur `actualites.json`. Ajoute un objet dans le tableau `articles` :

```json
{
  "id": "2026-10-12-coupe-atria",
  "titre": "Coupe Atria, samedi 21h",
  "categorie": "tournoi",
  "date": 1791876000,
  "resume": "Inscriptions ouvertes jusqu'a vendredi soir, en equipe de trois.",
  "langue": "fr",
  "epingle": false,
  "corps": "Le texte de l'annonce, en markdown."
}
```

- **`id`** : unique et stable, la date en tête rend la liste lisible.
- **`categorie`** : `tournoi`, `maps`, `creation`, `technique`, `entraide`. Autre chose est rangé dans « Autre » plutôt que de faire perdre l'article.
- **`date`** : secondes depuis 1970. [Convertisseur](https://www.epochconverter.com/).
- **`langue`** : `fr`, `en`, ou **vide**. Vide veut dire que l'article vaut dans les deux langues.
- **`epingle`** : passe devant les autres quelle que soit sa date. À utiliser rarement.
- **`corps`** : markdown. Titres, listes, citations, blocs de code, gras, italique, liens.

Ne touche pas à `actualites.json.sig`, et ne modifie pas `publie` : les deux sont refaits à la publication.

### Ce que le markdown accepte, et ses deux limites

Titres `#` à `###`, paragraphes, listes `-` et `1.`, citations `>`, blocs de code encadrés, séparateur `---`, et en ligne `**gras**`, `*italique*`, `` `code` `` et `[texte](https://…)`.

- **Les liens ne peuvent être que `http` ou `https`.** Un lien vers un fichier ou un programme s'affiche comme du texte, sans destination.
- **Les images deviennent des liens nommés.** Cairn n'autorise aucune image distante, donc une image resterait un cadre vide.

## La modération

C'est la revue de la proposition. Quelqu'un propose, un mainteneur lit et accepte ou refuse, et l'historique garde qui a écrit quoi et quand. Il n'y a pas d'autre outil, et il n'en faut pas d'autre.

## Publier (mainteneurs)

Une fois la proposition acceptée, mettre `publie` à l'horodatage du moment, puis depuis un dossier **en dehors** de tout dépôt :

```
cargo run --manifest-path <chemin>/cairn/crates/cairn-core/Cargo.toml --example actualites -- signer cairn-actualites-privee.txt actualites.json
```

Puis déposer **`actualites.json` d'abord, `actualites.json.sig` ensuite**. L'ordre inverse laisse une fenêtre pendant laquelle la signature ne correspond pas au fil, et Cairn refuse tout et garde son cache.

`publie` doit toujours augmenter : Cairn refuse un fil antérieur à celui qu'il a déjà, ce qui empêche de reservir un vieux fil signé pour faire disparaître une annonce.

## Licence

Les articles sont publiés sous [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/deed.fr), sauf mention contraire dans l'article.
