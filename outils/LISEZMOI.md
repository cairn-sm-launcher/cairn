# Outils

## `cle-de-developpement-PUBLIQUE-sans-valeur.txt`

**Ce fichier n'est pas un secret, et son nom le dit exprès.**

C'est la moitié privée d'un trousseau de signature **d'essai**, fabriqué pour
que l'échantillon de `docs/actualites/` soit signé et que la chaîne de
publication se teste de bout en bout sans dépendre de quoi que ce soit en
ligne. Elle est dans le dépôt, donc connue de tout le monde, donc elle ne
prouve rien et ne protège rien.

La clé qui compte, celle qui signe le vrai fil d'actualités, est fabriquée sur
la machine de celui qui publie et **n'entre jamais ici**. Le `.gitignore` du
projet écarte les noms habituels d'un fichier de clé, mais un `.gitignore` ne
rattrape pas tout : la règle est de ne jamais déposer une clé privée dans un
dépôt, pas de compter sur un filet.

Voir `docs/actualites/LISEZMOI.md` pour la fabrication du trousseau et la
publication.
