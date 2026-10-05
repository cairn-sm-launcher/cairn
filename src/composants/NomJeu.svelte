<!--
  Un texte venu du jeu, avec ses couleurs.

  Les noms de serveurs portent les codes de mise en forme de Trackmania,
  et leur proprietaire les a choisis en regardant le jeu, pas Cairn. Le
  decoupage et la correction de contraste sont faits en Rust, ou ils sont
  testes, voir `cairn_core::texte` et `cairn_core::lisibilite`. Ce
  composant ne fait que poser les morceaux.

  **Il rend toujours quelque chose.** Le decoupage arrive par le pont, et
  un pont a deux bouts qui peuvent ne pas etre de la meme version : c'est
  arrive, un binaire Rust d'avant le lot 20 avec l'interface d'apres, et
  la colonne des noms s'est videe entierement sans une ligne en console.
  Un `{#each}` sur un champ absent ne rend rien et ne se plaint pas. Le
  nom nu est donc toujours passe a cote du decoupage, et c'est lui qui
  s'affiche des que le decoupage manque ou que le joueur a coupe les
  couleurs. Une colonne sans couleurs reste utilisable, une colonne vide
  n'est rien.

  **Rien n'est construit a partir d'une chaine.** Les noms sont ecrits
  par des inconnus : chaque morceau est un noeud de texte, et la couleur
  est un `#RRGGBB` que Rust a fabrique a partir de trois chiffres
  hexadecimaux, jamais une chaine venue du reseau.
-->
<script lang="ts">
  import { reglages } from '../lib/etat.svelte';
  import type { SegmentAffiche } from '../lib/pont';

  let {
    segments,
    nu,
  }: {
    /** Le decoupage colore. Absent ou vide, c'est `nu` qui s'affiche. */
    segments?: SegmentAffiche[];
    /** Le nom debarrasse de ses codes. Toujours renseigne. */
    nu: string;
  } = $props();

  const colore = $derived(reglages.couleurs_jeu && !!segments && segments.length > 0);
</script>

{#if colore}{#each segments ?? [] as segment, rang (rang)}<span
      style:color={segment.couleur ?? undefined}
      style:font-weight={segment.gras ? '700' : undefined}
      style:font-style={segment.italique ? 'italic' : undefined}>{segment.texte}</span
    >{/each}{:else}{nu}{/if}
