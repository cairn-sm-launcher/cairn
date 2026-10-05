<!--
  La question que pose la croix de la fenetre.

  Trois precautions, et chacune repare une facon connue de pieger les
  gens.

  **La case est decochee.** Quelqu'un qui repond vite a une question
  qu'il voit pour la premiere fois ne doit pas s'engager pour toujours
  sans l'avoir voulu. Cochee par defaut, elle transforme un reflexe en
  reglage permanent.

  **Le bouton qui ferme n'a pas le focus.** Entree et Espace tombent donc
  sur « Reduire », qui est reversible. Fermer ne l'est pas : la session
  est perdue, et il faut rouvrir.

  **Echap annule.** Ni fermer ni reduire : on revient a ce qu'on faisait.
  Une question posee par surprise doit toujours avoir une sortie neutre.

  Et le reglage reste modifiable dans Reglages, section Application,
  sans quoi celui qui a coche la case serait prisonnier de son choix.
-->
<script lang="ts">
  import Bouton from './Bouton.svelte';
  import Icone from '../lib/Icone.svelte';
  import { t } from '../lib/traduction.svelte';
  import { reglages, ui, systeme, repondreFermeture } from '../lib/etat.svelte';

  let memoriser = $state(false);
  let reduireBouton: HTMLElement | undefined = $state();

  const accent = $derived(reglages.accent);

  // Le focus part sur l'option reversible, jamais sur celle qui ferme.
  $effect(() => {
    reduireBouton?.querySelector('button')?.focus();
  });

  function annuler() {
    ui.fermetureDemandee = false;
  }

  function repondre(choix: 'reduire' | 'quitter') {
    ui.fermetureDemandee = false;
    repondreFermeture(choix, memoriser);
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && annuler()} />

<div class="voile fade-in" onclick={annuler} role="presentation">
  <div
    class="carte"
    role="dialog"
    aria-modal="true"
    aria-label={t('Close Cairn?')}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      // Echap d'abord, l'arret de propagation ensuite. La carte coupe le
      // clavier pour que la navigation par onglets ne s'echappe pas
      // derriere elle, et cette coupure empechait la fenetre de voir la
      // touche qui annule : la sortie neutre etait murée.
      if (e.key === 'Escape') annuler();
      e.stopPropagation();
    }}
    tabindex="-1"
  >
    <div class="titre display">{t('Close Cairn?')}</div>
    <p class="sous">
      {t('Minimizing keeps Cairn in the taskbar. Closing ends it.')}
      <!-- Le jeu n'est jamais touche, quoi qu'on reponde. Le dire au
           moment ou la question se pose vaut mieux que de l'ecrire dans
           une aide que personne n'ouvre. -->
      {#if systeme.jeuLance}
        <span class="rassure">{t('ManiaPlanet keeps running either way.')}</span>
      {/if}
    </p>

    <label class="memo">
      <input type="checkbox" bind:checked={memoriser} />
      <span>{t('Remember my choice')}</span>
      <span class="ou">{t('Changeable in Settings, Application')}</span>
    </label>

    <div class="actions">
      <span bind:this={reduireBouton}>
        <Bouton libelle={t('Minimize')} {accent} onclick={() => repondre('reduire')} />
      </span>
      <Bouton
        libelle={t('Close')}
        genre="fantome"
        {accent}
        onclick={() => repondre('quitter')}
      />
      <button class="annuler" type="button" onclick={annuler}>
        <Icone nom="x" taille={14} epaisseur={2} />
        {t('Cancel')}
      </button>
    </div>
  </div>
</div>

<style>
  .voile {
    position: absolute;
    inset: 0;
    z-index: 36;
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
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 15px;
    padding: 22px 24px 20px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }
  .titre {
    font-size: 17px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .sous {
    font-size: 12.5px;
    color: #9aa3ad;
    line-height: 1.55;
    margin: 6px 0 16px;
  }
  .rassure {
    display: block;
    margin-top: 4px;
    color: #838c96;
  }
  .memo {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font-size: 12.5px;
    color: #c9d1d9;
    cursor: pointer;
    margin-bottom: 18px;
  }
  .memo input {
    width: 15px;
    height: 15px;
    accent-color: var(--accent, #20e3b2);
    cursor: pointer;
  }
  .ou {
    font-size: 11.5px;
    color: #6e7681;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .annuler {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    background: transparent;
    border: 0;
    color: #838c96;
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    padding: 8px 4px;
  }
  .annuler:hover {
    color: #f4f6f8;
  }
</style>
