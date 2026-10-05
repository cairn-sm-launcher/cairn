<!--
  La pastille d'un palier de rang.

  Elle sert au Classement, a l'Historique, et servira au profil et au
  lobby. Un seul composant, donc une seule idee de ce a quoi ressemble
  un palier : trois copies auraient fini par diverger, et un rang qui
  change d'aspect d'un ecran a l'autre cesse d'etre reconnaissable.

  Deux tailles. **Ample** porte le nom en toutes lettres, pour la tete du
  Classement ou le palier est le sujet. **Compacte** ne garde que le
  losange colore et son initiale, pour une colonne qu'on balaye : a cette
  taille c'est la couleur qui parle, pas le mot.
-->
<script lang="ts">
  import { PALIERS, POINTS_PAR_PALIER } from '../lib/constantes';
  import { t } from '../lib/traduction.svelte';

  let {
    palier,
    points,
    compact = false,
    etire = false,
  }: {
    /** L'indice du palier, de 0 pour Bronze a 4 pour Diamant. */
    palier: number;
    /** La progression dans le palier. Absente, la jauge ne s'affiche pas. */
    points?: number;
    compact?: boolean;
    /**
     * La jauge prend toute la largeur offerte au lieu de ses 108 px.
     *
     * Sert a la colonne Rang du Classement, ou la jauge occupe l'espace
     * qui restait vide entre le pseudo et les chiffres. Ce n'est pas du
     * remplissage : elle dit ou en est chacun dans son palier, donc qui
     * est sur le point de monter, ce qu'aucune autre colonne ne dit.
     */
    etire?: boolean;
  } = $props();

  const p = $derived(PALIERS[Math.max(0, Math.min(PALIERS.length - 1, palier))]);
  const part = $derived(
    points === undefined ? null : Math.max(0, Math.min(100, (points / POINTS_PAR_PALIER) * 100)),
  );
</script>

<div class="palier" class:compact class:etire title={compact ? t(p.libelle) : undefined}>
  <span class="losange" style="--teinte:{p.couleur};">
    <span class="lettre display">{t(p.libelle).charAt(0)}</span>
  </span>
  {#if !compact}
    <span class="texte">
      <span class="nom display" style="color:{p.couleur};">{t(p.libelle)}</span>
      {#if part !== null}
        <span class="jauge">
          <span class="rempli" style="width:{part}%; background:{p.couleur};"></span>
        </span>
      {/if}
    </span>
  {/if}
</div>

<style>
  .palier {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  /* Un losange et non un rond : a douze pixels dans une colonne, la
     silhouette se distingue d'une pastille d'activite ou d'une etoile de
     favori, qui sont rondes. La forme porte autant que la couleur. */
  .losange {
    width: 22px;
    height: 22px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transform: rotate(45deg);
    border-radius: 5px;
    background: color-mix(in srgb, var(--teinte) 18%, transparent);
    border: 1px solid var(--teinte);
  }
  .lettre {
    transform: rotate(-45deg);
    font-size: 11px;
    font-weight: 700;
    color: var(--teinte);
    line-height: 1;
  }
  .texte {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .nom {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
  }
  .jauge {
    display: block;
    width: 108px;
    height: 4px;
    border-radius: 3px;
    background: rgba(255, 255, 255, 0.09);
    overflow: hidden;
  }
  .rempli {
    display: block;
    height: 100%;
    border-radius: 3px;
  }
  .etire,
  .etire .texte {
    flex: 1;
  }
  .etire .jauge {
    width: auto;
  }
  .compact .losange {
    width: 18px;
    height: 18px;
    border-radius: 4px;
  }
  .compact .lettre {
    font-size: 9.5px;
  }
</style>
