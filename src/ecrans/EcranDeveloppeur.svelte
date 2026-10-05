<!--
  L'ecran Developpeur : le lecteur du journal, et de quoi le transmettre.

  **Il sert d'abord a quelqu'un d'autre que le developpeur.** Flow a un
  testeur depuis le 2026-09-28, et c'est lui le vrai destinataire : cet
  ecran est ce qui transforme « ca marche pas » en une ligne datee avec
  un message. D'ou le fait que le bouton qui copie un rapport soit aussi
  visible que le journal lui meme.

  **Il vaut aussi comme preuve de bonne foi.** Cairn ecrit un fichier sur
  la machine de quelqu'un ; ouvrir ce fichier en lecture, dans
  l'application, sans rien cacher, est la facon la plus simple de montrer
  ce qu'il contient. C'est pour ca que l'ecran ne vit pas derriere un
  interrupteur « mode developpeur » : il n'y a rien a deverrouiller.

  Ce qui n'est PAS ici, et qui vit en Rust parce que ca merite des tests :
  le decoupage des lignes, le classement par niveau, l'anonymisation, les
  comptes, la recherche et la limite. Voir `cairn_core::journal`. Cet
  ecran affiche et n'interprete rien.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Interrupteur from '../composants/Interrupteur.svelte';
  import { t } from '../lib/traduction.svelte';
  import { reglages, systeme } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';
  import type { NiveauJournal, VueJournal } from '../lib/pont';
  import { onDestroy } from 'svelte';

  const accent = $derived(reglages.accent);

  /**
   * Combien de lignes on affiche au plus.
   *
   * Le journal monte a un mega-octet avant d'etre archive, et l'archive
   * est lue avec lui : ca fait plus de vingt mille lignes, donc plus de
   * vingt mille noeuds dans la page. Rust rend les mille dernieres et dit
   * combien manquent, avec un bouton pour tout demander.
   */
  const LIMITE = 1000;

  /** Toutes les deux secondes, et seulement pendant le suivi. */
  const PERIODE = 2000;

  const COULEURS: Record<NiveauJournal, string> = {
    info: '#6F7883',
    alerte: '#FFC857',
    erreur: '#E3706A',
  };

  let niveaux = $state<NiveauJournal[]>(['info', 'alerte', 'erreur']);
  let recherche = $state('');
  let anonyme = $state(true);
  let limite = $state(LIMITE);
  let vue = $state<VueJournal | null>(null);
  let erreur = $state<string | null>(null);
  let note = $state<string | null>(null);

  /**
   * Le suivi du bas, qui commande aussi le rechargement.
   *
   * Les deux sont volontairement le meme interrupteur. Recharger le
   * journal pendant que quelqu'un lit une ligne du milieu le lui ferait
   * sauter sous les yeux ; arreter de recharger quand il remonte regle
   * la pause au defilement et la relecture d'un coup, sans deuxieme
   * mecanisme a comprendre.
   */
  let suivi = $state(true);
  let liste: HTMLElement | undefined = $state();

  async function recharger() {
    try {
      vue = await pont.lireJournal(niveaux, recherche, anonyme, limite);
      erreur = null;
    } catch (e) {
      erreur = String(e);
    }
  }

  // Le filtre, la recherche et l'anonymisation repartent tout de suite :
  // une barre de recherche qui attend deux secondes passe pour cassee.
  $effect(() => {
    void niveaux;
    void recherche;
    void anonyme;
    void limite;
    recharger();
  });

  const minuterie = setInterval(() => {
    if (suivi) recharger();
  }, PERIODE);
  onDestroy(() => clearInterval(minuterie));

  // Apres chaque rendu de la liste, on recolle en bas tant qu'on suit.
  $effect(() => {
    void vue;
    if (suivi && liste) liste.scrollTop = liste.scrollHeight;
  });

  /** Un defilement vers le haut coupe le suivi, un retour en bas le rend. */
  function auDefilement() {
    if (!liste) return;
    const enBas = liste.scrollHeight - liste.scrollTop - liste.clientHeight < 24;
    suivi = enBas;
  }

  function reprendre() {
    suivi = true;
    if (liste) liste.scrollTop = liste.scrollHeight;
  }

  function basculer(niveau: NiveauJournal) {
    niveaux = niveaux.includes(niveau)
      ? niveaux.filter((n) => n !== niveau)
      : [...niveaux, niveau];
  }

  const compte = (niveau: NiveauJournal) => vue?.comptes[niveau] ?? 0;

  /**
   * Les lignes, avec l'heure separee du jour.
   *
   * Premiere version : l'horodatage entier dans une colonne de 92 px. Il
   * se cassait en deux lignes, donc chaque entree occupait deux fois sa
   * hauteur et la liste devenait impossible a balayer. Elargir la colonne
   * a 140 px aurait repare l'affichage en gaspillant la place sur une
   * date qui se repete a l'identique des centaines de fois.
   *
   * Le jour n'est donc ecrit qu'au moment ou il change, dans un
   * separateur, comme dans n'importe quelle messagerie. On garde
   * l'information et on la paye une fois par journee au lieu d'une fois
   * par ligne.
   */
  const avecJours = $derived.by(() => {
    let jourPrecedent = '';
    return (vue?.lignes ?? []).map((ligne) => {
      const espace = ligne.horodatage.indexOf(' ');
      const jour = espace > 0 ? ligne.horodatage.slice(0, espace) : '';
      const heure = espace > 0 ? ligne.horodatage.slice(espace + 1) : '';
      const nouveau = jour !== '' && jour !== jourPrecedent;
      if (nouveau) jourPrecedent = jour;
      return { ligne, jour, heure, nouveau };
    });
  });

  /** Un message passager sous les boutons, efface au bout de quatre secondes. */
  let effacement: ReturnType<typeof setTimeout> | undefined;
  function annoncer(texte: string) {
    note = texte;
    clearTimeout(effacement);
    effacement = setTimeout(() => (note = null), 4000);
  }
  onDestroy(() => clearTimeout(effacement));

  async function copierLeRapport() {
    try {
      const texte = await pont.rapportJournal();
      await navigator.clipboard.writeText(texte);
      annoncer(t('Report copied. Paste it wherever you report the problem.'));
    } catch {
      // Le presse-papiers peut etre refuse par le systeme. Plutot que de
      // laisser croire que c'est copie, on dit quoi faire a la place.
      annoncer(t('Cairn could not reach the clipboard. Use Save report instead.'));
    }
  }

  async function enregistrerLeRapport() {
    try {
      const nom = await pont.enregistrerRapport();
      annoncer(t('Report saved as {0}, in the log folder.', nom));
    } catch (e) {
      annoncer(String(e));
    }
  }
</script>

<div class="ecran">
  <header class="pad entete">
    <div>
      <h1 class="titre display">{t('Developer')}</h1>
      <p class="sous-titre">
        {t('Everything Cairn writes about itself on this computer, as it writes it.')}
      </p>
    </div>
  </header>

  <!-- Signaler vient AVANT le journal, et ce n'est pas un accident.
       Quelqu'un qui ouvre cet ecran veut le plus souvent transmettre
       quelque chose, pas lire vingt mille lignes. Le journal est dessous,
       pour qui veut verifier ce qu'il envoie. -->
  <div class="pad">
    <div class="rapport">
      <div class="texte">
        <h2 class="sous-section display">{t('Report a problem')}</h2>
        <p>
          {t(
            'The report holds this log plus the version, the engine contract and the system. Paths, addresses and e-mails are always replaced, even when the log below shows them.',
          )}
        </p>
      </div>
      <div class="boutons">
        <Bouton libelle={t('Copy report')} {accent} onclick={copierLeRapport} />
        <Bouton
          libelle={t('Save report')}
          genre="fantome"
          {accent}
          onclick={enregistrerLeRapport}
        />
        <Bouton
          libelle={t('Open folder')}
          genre="fantome"
          {accent}
          onclick={() => pont.ouvrirDossierJournal()}
        />
      </div>
      {#if note}
        <p class="note" style="border-color:{accent};">{note}</p>
      {/if}
    </div>
  </div>

  <div class="pad barre">
    <label class="chercher">
      <Icone nom="search" taille={15} couleur="#838C96" epaisseur={1.8} />
      <input
        class="champ"
        type="search"
        bind:value={recherche}
        placeholder={t('Search the log')}
        aria-label={t('Search the log')}
      />
    </label>

    <!-- Les comptes portent sur le fichier entier et pas sur ce qui est
         affiche : ils servent a dire ce que le journal contient, donc ils
         ne doivent pas tomber a zero quand on decoche une case. -->
    <div class="niveaux" role="group" aria-label={t('Filter by level')}>
      {#each ['erreur', 'alerte', 'info'] as const as niveau (niveau)}
        <button
          class="niveau"
          type="button"
          aria-pressed={niveaux.includes(niveau)}
          class:eteint={!niveaux.includes(niveau)}
          style="--teinte:{COULEURS[niveau]};"
          onclick={() => basculer(niveau)}
        >
          <span class="point"></span>
          <span>{t(niveau === 'erreur' ? 'Errors' : niveau === 'alerte' ? 'Warnings' : 'Info')}</span
          >
          <span class="combien mono">{compte(niveau)}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="pad">
    <div class="cadre">
      {#if erreur}
        <p class="vide">{erreur}</p>
      {:else if !vue}
        <p class="vide">{t('Loading...')}</p>
      {:else if vue.lignes.length === 0}
        <p class="vide">
          {vue.comptes.info + vue.comptes.alerte + vue.comptes.erreur === 0
            ? t('The log is empty. Nothing has been written yet.')
            : t('Nothing matches. Try another word, or turn a level back on.')}
        </p>
      {:else}
        <div class="liste mono" bind:this={liste} onscroll={auDefilement}>
          {#if vue.tronquees > 0}
            <div class="tronque">
              <span>{t('{0} older lines are not shown.', vue.tronquees)}</span>
              <button type="button" style="color:{accent};" onclick={() => (limite = 0)}>
                {t('Show everything')}
              </button>
            </div>
          {/if}
          {#each avecJours as entree (entree.ligne.numero)}
            {#if entree.nouveau}
              <div class="jour"><span>{entree.jour}</span></div>
            {/if}
            <div class="ligne" style="--teinte:{COULEURS[entree.ligne.niveau]};">
              <span class="rang">{entree.ligne.numero}</span>
              <span class="heure" title={entree.ligne.horodatage}>{entree.heure}</span>
              <span class="marque"></span>
              <span class="message">{entree.ligne.message}</span>
            </div>
          {/each}
        </div>
      {/if}

      <div class="pied">
        <label class="bascule">
          <Interrupteur
            actif={!anonyme}
            {accent}
            libelle={t('Show full paths')}
            onbasculer={() => (anonyme = !anonyme)}
          />
          <span>{t('Show full paths')}</span>
        </label>
        <span class="explication">
          {t('Only changes what you see here. The report is always anonymised.')}
        </span>
        {#if !suivi}
          <button class="reprendre" type="button" style="color:{accent};" onclick={reprendre}>
            <Icone nom="down" taille={14} couleur={accent} epaisseur={2} />
            <span>{t('Resume live')}</span>
          </button>
        {:else}
          <span class="direct"><span class="pastille" style="background:{accent};"></span>
            {t('Live')}</span>
        {/if}
      </div>
    </div>
  </div>

  <p class="pad bas">
    <Icone nom="info" taille={15} couleur="#838C96" epaisseur={1.7} />
    <span>
      {t(
        'Cairn {0}, engine contract {1}, on {2}.',
        systeme.diagnostic?.version ?? '?',
        systeme.diagnostic?.contrat ?? '?',
        systeme.diagnostic ? `${systeme.diagnostic.os} / ${systeme.diagnostic.arch}` : '?',
      )}
    </span>
  </p>
</div>

<style>
  .ecran {
    flex: 1;
    overflow-y: auto;
    padding-top: var(--air-haut, 24px);
    padding-bottom: 30px;
    position: relative;
    z-index: 1;
  }
  .entete {
    margin-bottom: 18px;
  }
  .titre {
    font-size: 21px;
    font-weight: 600;
    margin: 0;
  }
  .sous-titre {
    font-size: 12px;
    color: #838c96;
    margin: 3px 0 0;
  }
  .sous-section {
    font-size: 13px;
    font-weight: 600;
    margin: 0 0 5px;
  }

  .rapport {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 22px;
    padding: 16px 20px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    margin-bottom: 22px;
  }
  .rapport p {
    font-size: 12.5px;
    color: #9aa3ad;
    margin: 0;
    max-width: 68ch;
  }
  .boutons {
    display: flex;
    gap: 9px;
    flex-wrap: wrap;
  }
  .note {
    grid-column: 1 / -1;
    font-size: 12.5px;
    color: #c9d1d9 !important;
    border-left: 2px solid;
    padding-left: 11px;
  }

  .barre {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 12px;
  }
  .chercher {
    display: flex;
    align-items: center;
    gap: 9px;
    flex: 1 1 240px;
    max-width: 380px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 9px;
    padding: 0 11px;
    height: 34px;
  }
  .champ {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    outline: none;
    color: #f4f6f8;
    font-size: 12.5px;
  }
  .niveaux {
    display: flex;
    gap: 6px;
  }
  .niveau {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 12px;
    border-radius: 9px;
    border: 1px solid color-mix(in srgb, var(--teinte) 34%, transparent);
    background: color-mix(in srgb, var(--teinte) 11%, transparent);
    color: #e6e9ee;
    font-size: 12.5px;
    cursor: pointer;
  }
  /* Une case decochee s'efface, elle ne disparait pas : son compte reste
     lisible, c'est justement ce qu'on veut savoir avant de la recocher.
     Meme vocabulaire que les pastilles eteintes du bandeau. */
  .niveau.eteint {
    border-color: rgba(255, 255, 255, 0.09);
    background: transparent;
    color: #6f7883;
  }
  .point {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--teinte);
  }
  .niveau.eteint .point {
    background: #3a424c;
  }
  .combien {
    font-size: 11.5px;
    color: #9aa3ad;
  }
  .niveau.eteint .combien {
    color: #5b636d;
  }

  .cadre {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    background: #0d1014;
    overflow: hidden;
  }
  .liste {
    height: 52vh;
    min-height: 260px;
    overflow-y: auto;
    padding: 8px 0;
  }
  .ligne {
    display: grid;
    grid-template-columns: 52px 78px 3px minmax(0, 1fr);
    align-items: baseline;
    gap: 10px;
    padding: 2px 14px;
    font-size: 11.5px;
    line-height: 1.55;
  }
  .ligne:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .rang {
    color: #4d555e;
    text-align: right;
  }
  .heure {
    color: #6f7883;
    white-space: nowrap;
  }
  /* Le jour ne s'ecrit qu'au moment ou il change. Un filet plutot qu'un
     bandeau : il separe sans se faire passer pour une entree. */
  .jour {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 10px 14px 6px;
    font-size: 10.5px;
    letter-spacing: 0.06em;
    color: #5b636d;
  }
  .jour::after {
    content: '';
    flex: 1;
    height: 1px;
    background: rgba(255, 255, 255, 0.07);
  }
  /* Un filet et pas une pastille : a cette densite, vingt pastilles
     rondes dans une colonne font du bruit, alors qu'un filet suit la
     ligne et se lit comme une marge. */
  .marque {
    align-self: stretch;
    border-radius: 2px;
    background: var(--teinte);
  }
  .message {
    color: #c9d1d9;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .tronque {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 14px 12px;
    font-size: 11.5px;
    color: #838c96;
    border-bottom: 1px dashed rgba(255, 255, 255, 0.1);
    margin-bottom: 6px;
  }
  .tronque button {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    cursor: pointer;
    text-decoration: underline;
  }
  .vide {
    padding: 40px 18px;
    font-size: 12.5px;
    color: #838c96;
    margin: 0;
  }

  .pied {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 10px 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.07);
    background: rgba(255, 255, 255, 0.02);
    flex-wrap: wrap;
  }
  .bascule {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: #c9d1d9;
  }
  .explication {
    flex: 1;
    min-width: 0;
    font-size: 11.5px;
    color: #6f7883;
  }
  .reprendre,
  .direct {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 11.5px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    white-space: nowrap;
  }
  .direct {
    color: #838c96;
    cursor: default;
  }
  .pastille {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .bas {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: 18px;
    font-size: 11.5px;
    color: #838c96;
  }
  @media (max-width: 900px) {
    .rapport {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
