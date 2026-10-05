<!--
  Un groupe de choix exclusifs, deux a quatre options cote a cote.

  Il sert a la langue, a l'intensite du fond et a sa vitesse. Au dela de
  quatre options il faudrait autre chose : la rangee deborderait et le
  choix deviendrait un balayage.
-->
<script lang="ts">
  let {
    options,
    choix,
    accent,
    libelle,
    onchoisir,
    onapercu,
  }: {
    options: Array<{ id: string; libelle: string }>;
    choix: string;
    accent: string;
    libelle: string;
    onchoisir: (id: string) => void;
    /**
     * Le survol d'une option, et `null` quand on le quitte.
     *
     * Facultatif, et un seul appelant s'en sert : la densite, qui montre
     * a cote ce que l'option survolee donnerait. Le clavier compte
     * autant que la souris, d'ou le focus en plus du survol.
     */
    onapercu?: (id: string | null) => void;
  } = $props();
</script>

<div class="groupe" role="group" aria-label={libelle}>
  {#each options as option (option.id)}
    <button
      class="seg"
      type="button"
      aria-pressed={choix === option.id}
      onclick={() => onchoisir(option.id)}
      onmouseenter={() => onapercu?.(option.id)}
      onmouseleave={() => onapercu?.(null)}
      onfocus={() => onapercu?.(option.id)}
      onblur={() => onapercu?.(null)}
      style={choix === option.id ? `background:${accent};color:#0A0C0F;` : ''}
    >
      {option.libelle}
    </button>
  {/each}
</div>

<style>
  .groupe {
    display: inline-flex;
    gap: 3px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    padding: 3px;
    flex-shrink: 0;
  }
  .seg {
    padding: 7px 15px;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: #9aa3ad;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  .seg:hover {
    color: #f4f6f8;
  }
</style>
