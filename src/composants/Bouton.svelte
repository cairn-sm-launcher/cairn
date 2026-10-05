<!--
  Les deux boutons de texte de l'interface.

  `principal` porte la couleur d'accent sur fond plein, `fantome` n'a
  qu'un contour. Un ecran n'a qu'un seul bouton principal a la fois,
  c'est ce qui fait qu'on le repere sans le chercher.
-->
<script lang="ts">
  let {
    libelle,
    genre = 'principal',
    accent,
    onclick,
  }: {
    libelle: string;
    genre?: 'principal' | 'fantome';
    accent: string;
    onclick?: () => void;
  } = $props();
</script>

<button
  class={genre === 'principal' ? 'btn-cta principal' : 'btn-ghost fantome'}
  type="button"
  {onclick}
  style={genre === 'principal' ? `background:${accent};` : ''}
>
  {libelle}
</button>

<style>
  button {
    border-radius: 10px;
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  .principal {
    color: #0a0c0f;
    border: none;
    padding: 11px 22px;
  }
  .fantome {
    background: transparent;
    color: #f4f6f8;
    border: 1px solid rgba(255, 255, 255, 0.14);
    padding: 11px 20px;
  }
</style>
