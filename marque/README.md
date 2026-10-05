# La marque Cairn

Un cube isométrique dans un hexagone, dessiné par Xirdrak sous Illustrator.
Dégradé `#1fdbac` vers `#80d5ff`, la même famille teal que l'accent de l'interface.

| Fichier | Rôle |
|---|---|
| `cairn.svg` | La référence. Plan de travail 500 par 500, sans calque de construction. C'est lui qui sert à générer les icônes. |
| `cairn-16.svg` | La déclinaison dessinée pour les petites tailles. Même tracé, contours un peu plus épais. Sert aux entrées 16, 20 et 24 px du `.ico`. |
| `cairn-source-illustrator.svg` | L'export Illustrator d'origine, calque `Calque_1` compris. C'est la grille isométrique de construction, cachée, qui pèse 979 des 986 Ko du fichier. On la garde ici pour pouvoir reprendre le dessin, jamais dans un livrable. |

## Régénérer les icônes

```
python3 marque/generer-icones.py
```

Le script écrit dans `src-tauri/icons/`. Il prend `cairn.svg` à partir de 32 px
et `cairn-16.svg` en dessous de 24 px, parce que le double contour extérieur
n'a pas assez de pixels pour exister en dessous de cette taille.

## Dans l'interface

La marque est inlinée dans `src/App.svelte`, pas chargée comme fichier. Deux
raisons : la politique de sécurité du contenu n'a rien à autoriser de plus, et
un SVG inline hérite du thème. Les identifiants de dégradés y sont préfixés
`cairn` pour ne pas entrer en conflit avec d'autres SVG de la page, les
identifiants d'Illustrator étant accentués et donc fragiles.
