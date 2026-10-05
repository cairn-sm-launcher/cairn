<!--
  L'ecran d'un onglet qui existe mais n'a pas encore son contenu.

  Il dit ce qui viendra et quand, plutot qu'un « indisponible » sec. Un
  espace vide sans explication passe pour une panne, une phrase datee
  passe pour un chantier.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';

  // Le titre et l'explication arrivent deja traduits. Les traduire ici
  // reviendrait a passer une cle par variable, et verifier-traduction.mjs
  // ne voit que les chaines litterales : l'oubli passerait inapercu.
  let {
    icone,
    titre,
    explication,
  }: {
    icone: string;
    titre: string;
    explication: string;
  } = $props();
</script>

<div class="pad ecran">
  <div class="carte">
    <Icone nom={icone} taille={34} couleur="#838C96" epaisseur={1.5} />
    <div class="titre display">{titre}</div>
    <p class="explication">{explication}</p>
  </div>
</div>

<style>
  .ecran {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding-bottom: 60px;
    position: relative;
    z-index: 1;
  }
  .carte {
    max-width: 460px;
    text-align: center;
    background: #13161b;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 16px;
    padding: 36px 34px;
  }
  .titre {
    font-weight: 700;
    font-size: 19px;
    letter-spacing: -0.02em;
    margin-top: 16px;
  }
  .explication {
    font-size: 13px;
    color: #9aa3ad;
    line-height: 1.6;
    margin: 10px 0 0;
  }
</style>
