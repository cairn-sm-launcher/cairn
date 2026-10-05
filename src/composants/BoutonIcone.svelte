<!--
  Le bouton carre de 38 px qui ne porte qu'une icone.

  Il sert dans le bandeau pour la cloche, les amis et l'engrenage. Quand
  il est actif, sa bordure et son icone prennent la couleur d'accent, ce
  qui evite un fond plein de plus dans une rangee qui en compte deja.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';

  let {
    nom,
    titre,
    actif = false,
    desactive = false,
    accent,
    onclick,
    children,
  }: {
    nom: string;
    titre: string;
    actif?: boolean;
    /**
     * Present mais inerte. Sert a la fleche Precedent du bandeau quand
     * il n'y a rien derriere : elle reste a sa place, eteinte, plutot
     * que de disparaitre et de decaler ses voisines.
     */
    desactive?: boolean;
    accent: string;
    onclick?: () => void;
    children?: import('svelte').Snippet;
  } = $props();
</script>

<button
  class="icon-btn"
  type="button"
  title={titre}
  aria-label={titre}
  disabled={desactive}
  {onclick}
  style="border-color:{actif ? accent : 'rgba(255,255,255,0.08)'};color:{actif
    ? accent
    : '#9AA3AD'};"
>
  <Icone {nom} epaisseur={1.7} />
  {@render children?.()}
</button>

<style>
  button {
    width: 38px;
    height: 38px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    position: relative;
    flex-shrink: 0;
  }
  button:disabled {
    opacity: 0.32;
    cursor: default;
  }
</style>
