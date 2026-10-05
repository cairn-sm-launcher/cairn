<!--
  L'interrupteur des reglages.

  Un bouton, pas une case a cocher habillee : il porte aria-pressed, donc
  un lecteur d'ecran annonce son etat, et il repond a Espace comme a
  Entree sans qu'on ait rien a ecrire.
-->
<script lang="ts">
  let {
    actif,
    accent,
    libelle,
    onbasculer,
  }: {
    actif: boolean;
    accent: string;
    libelle: string;
    onbasculer: () => void;
  } = $props();
</script>

<button
  class="interrupteur"
  type="button"
  role="switch"
  aria-checked={actif}
  aria-label={libelle}
  onclick={onbasculer}
  style="background:{actif ? accent : 'rgba(255,255,255,0.08)'};border-color:{actif
    ? accent
    : 'rgba(255,255,255,0.16)'};"
>
  <span
    class="pion"
    style="background:{actif ? '#0A0C0F' : '#9AA3AD'};margin-left:{actif ? '20px' : '0'};"
  ></span>
</button>

<style>
  .interrupteur {
    width: 46px;
    height: 26px;
    border-radius: 999px;
    border: 1px solid;
    display: flex;
    align-items: center;
    cursor: pointer;
    padding: 2px;
    flex-shrink: 0;
  }
  .pion {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    transition: margin-left 0.2s cubic-bezier(0.2, 0.7, 0.3, 1);
  }
</style>
