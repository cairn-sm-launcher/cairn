<!--
  Le profil actif. Un clic ouvre le selecteur.

  Il vivait a gauche du bandeau, il est passe a droite a l'etape 30, avec
  les notifications, le chat et les reglages. Le deplacement n'est pas
  qu'une affaire de position : a gauche il etait l'ancre de l'ecran et
  avait la place de respirer, a droite il est un bouton parmi quatre.

  Il porte donc deux tailles. **Ample**, avec son sous-titre et son
  chevron, pour le selecteur de profil et l'assistant, ou il est le sujet
  de l'ecran. **Compact**, la pastille et le nom seuls, pour le bandeau :
  le sous-titre « Profil local » y repeterait a longueur de journee une
  information qu'on lit une fois, et le chevron promet un menu deroulant
  alors que le clic ouvre une surimpression.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import { t } from '../lib/traduction.svelte';
  import { reglages, ui, systeme } from '../lib/etat.svelte';

  let { compact = false }: { compact?: boolean } = $props();

  const profil = $derived(
    systeme.profils.find((p) => p.id === reglages.profil) ?? systeme.profils[0],
  );
</script>

<button
  class="profile-btn"
  class:compact
  type="button"
  title={t('Switch profile')}
  onclick={() => (ui.selecteurProfil = true)}
>
  <span class="pastille display" style="background:{reglages.accent};">
    {profil?.initiale ?? '?'}
  </span>
  <span class="texte">
    <span class="nom display">{profil?.nom ?? t('No profile')}</span>
    {#if !compact}<span class="sous">{t('Local profile')}</span>{/if}
  </span>
  {#if !compact}
    <span class="chev"><Icone nom="chev" taille={15} epaisseur={2} /></span>
  {/if}
</button>

<style>
  button {
    display: flex;
    align-items: center;
    gap: 13px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 12px;
    padding: 6px 10px 6px 6px;
    cursor: pointer;
    color: #f4f6f8;
    text-align: left;
  }
  .pastille {
    width: 40px;
    height: 40px;
    border-radius: 11px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 17px;
    color: #0a0c0f;
    flex-shrink: 0;
  }
  .nom {
    display: block;
    font-weight: 600;
    font-size: 15px;
  }
  .sous {
    display: block;
    font-size: 12px;
    color: #838c96;
  }
  .chev {
    opacity: 0.35;
    transition: opacity 0.15s ease;
    color: #838c96;
    display: flex;
  }

  /* La version du bandeau. La pastille descend a la taille des boutons
     icone qui la suivent, sans quoi elle ferait une tete de plus qu'eux
     et casserait la ligne du groupe. */
  .compact {
    gap: 9px;
    padding: 4px 8px 4px 4px;
  }
  .compact .pastille {
    width: 30px;
    height: 30px;
    border-radius: 9px;
    font-size: 13px;
  }
  .compact .nom {
    font-size: 13px;
    max-width: 130px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* En fenetre etroite il ne reste que la pastille. Elle suffit : la
     couleur et l'initiale sont ce qui fait reconnaitre un profil du coin
     de l'oeil, le nom n'est qu'une confirmation. */
  @media (max-width: 1150px) {
    .compact .texte {
      display: none;
    }
  }
</style>
