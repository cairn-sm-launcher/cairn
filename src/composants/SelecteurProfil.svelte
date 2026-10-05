<!--
  Le selecteur de profil, en surimpression.

  Un clic sur le voile ferme, un clic dans la carte non : c'est pour ca
  que la carte arrete la propagation plutot que de compter sur la cible
  de l'evenement.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import LigneProfil from './LigneProfil.svelte';
  import { t } from '../lib/traduction.svelte';
  import { reglages, ui, systeme, enregistrer } from '../lib/etat.svelte';

  function fermer() {
    ui.selecteurProfil = false;
  }

  function choisir(id: string) {
    reglages.profil = id;
    enregistrer();
    fermer();
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && fermer()} />

<!-- Le voile est un fond cliquable, pas un bouton : il porte donc un
     role de presentation et la fermeture reste accessible au clavier par
     la touche Echap et par le bouton de fermeture. -->
<div class="voile fade-in" onclick={fermer} role="presentation">
  <div
    class="carte"
    role="dialog"
    aria-modal="true"
    aria-label={t('Switch profile')}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
    tabindex="-1"
  >
    <div class="entete">
      <div>
        <div class="titre display">{t('Switch profile')}</div>
        <div class="sous">{t('Profiles stored by ManiaPlanet on this computer')}</div>
      </div>
      <button class="icon-btn fermer" type="button" title={t('Close')} onclick={fermer}>
        <Icone nom="x" taille={15} epaisseur={2} />
      </button>
    </div>

    <div class="liste">
      {#each systeme.profils as profil (profil.id)}
        <LigneProfil
          {profil}
          choisi={reglages.profil === profil.id}
          accent={reglages.accent}
          onclick={() => choisir(profil.id)}
        />
      {/each}
      {#if systeme.profils.length === 0}
        <div class="vide">{t('No profile found on this computer')}</div>
      {/if}
    </div>

    <div class="pied">
      <Icone nom="info" taille={15} couleur="#838C96" epaisseur={1.7} />
      <span>{t('Close the game before switching, ManiaPlanet loads a profile at startup.')}</span>
    </div>
  </div>
</div>

<style>
  .voile {
    position: absolute;
    inset: 0;
    z-index: 30;
    background: rgba(10, 12, 15, 0.74);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 30px;
  }
  .carte {
    width: 100%;
    max-width: 430px;
    background: #13161b;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    padding: 22px 22px 18px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
  }
  .entete {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .titre {
    font-weight: 700;
    font-size: 17px;
    letter-spacing: -0.02em;
  }
  .sous {
    font-size: 12px;
    color: #838c96;
    margin-top: 3px;
  }
  .fermer {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    border: none;
    background: transparent;
    color: #838c96;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }
  .liste {
    margin-top: 18px;
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  .vide {
    font-size: 12.5px;
    color: #838c96;
    padding: 12px 2px;
  }
  .pied {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 16px;
    padding-top: 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.07);
    font-size: 11.5px;
    color: #838c96;
    line-height: 1.5;
  }
</style>
