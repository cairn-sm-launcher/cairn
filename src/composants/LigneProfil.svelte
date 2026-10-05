<!--
  Une ligne de profil, dans l'assistant comme dans le selecteur.

  La pastille prend son initiale du nom et sa couleur d'une teinte tiree
  de ce meme nom : deux profils differents n'ont jamais la meme, et le
  meme profil garde la sienne d'un lancement a l'autre.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import { noteProfil } from '../lib/traduction.svelte';
  import type { Profil } from '../lib/pont';

  let {
    profil,
    choisi,
    accent,
    onclick,
  }: {
    profil: Profil;
    choisi: boolean;
    accent: string;
    onclick: () => void;
  } = $props();

  const note = $derived(noteProfil(profil.derniere_utilisation));
</script>

<button
  class="pick-row"
  type="button"
  {onclick}
  style="background:{choisi
    ? 'rgba(255,255,255,0.05)'
    : 'transparent'};border-color:{choisi ? accent : 'rgba(255,255,255,0.09)'};"
>
  <span class="pastille" style="background:{profil.teinte};">{profil.initiale}</span>
  <span class="texte">
    <span class="nom">{profil.nom}</span>
    {#if note}<span class="note">{note}</span>{/if}
  </span>
  {#if choisi}
    <span class="coche" style="color:{accent};">
      <Icone nom="check" taille={17} epaisseur={2.2} />
    </span>
  {/if}
</button>

<style>
  button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 13px;
    padding: 13px 15px;
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
    color: #f4f6f8;
    border: 1px solid rgba(255, 255, 255, 0.09);
  }
  .pastille {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    color: #0a0c0f;
    font-weight: 700;
    font-size: 13.5px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .texte {
    flex: 1;
    min-width: 0;
  }
  .nom {
    display: block;
    font-size: 13.5px;
    font-weight: 600;
  }
  .note {
    display: block;
    font-size: 11.5px;
    color: #838c96;
    margin-top: 2px;
  }
  .coche {
    display: flex;
    flex-shrink: 0;
  }
</style>
