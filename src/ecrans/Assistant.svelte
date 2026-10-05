<!--
  L'assistant de premier lancement, trois etapes.

  Pendant qu'il est a l'ecran, seule l'ambiance reste derriere la carte.
  Pas de bandeau, pas d'encart : l'ecran ne fait qu'une chose a la fois.

  La hauteur du corps est FIGEE et pas seulement minimale. Les quatre
  etats mesuraient 252, 336, 323 et 252 px : la carte etant centree, ses
  boutons se deplaçaient a chaque etape. On cale sur le plus grand des
  quatre et la carte ne bouge plus jamais.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import LigneProfil from '../composants/LigneProfil.svelte';
  import Marque from '../composants/Marque.svelte';
  import { t } from '../lib/traduction.svelte';
  import { ACCENTS, CHEMINS_COURANTS } from '../lib/constantes';
  import { rgba } from '../lib/couleurs';
  import {
    reglages,
    ui,
    systeme,
    enregistrer,
    enregistrerMaintenant,
    releverEtat,
  } from '../lib/etat.svelte';

  const accent = $derived(reglages.accent);

  /** L'etape 1 bascule en saisie de chemin quand on le lui demande. */
  let saisieChemin = $state(false);
  let brouillon = $state('');

  function ouvrirSaisie() {
    brouillon = reglages.chemin_jeu ?? systeme.cheminJeu ?? '';
    saisieChemin = true;
  }

  async function validerChemin() {
    const propre = brouillon.trim();
    reglages.chemin_jeu = propre === '' ? null : propre;
    saisieChemin = false;
    // Rust garde les reglages en memoire, donc il doit avoir recu le
    // chemin avant qu'on lui redemande l'etat. D'ou l'enregistrement
    // immediat plutot que celui qui attend quatre cents millisecondes.
    await enregistrerMaintenant().catch(() => {});
    await releverEtat().catch(() => {});
  }

  function suivant() {
    if (ui.etape < 3) {
      ui.etape += 1;
      return;
    }
    terminer();
  }

  function terminer() {
    if (ui.choix) reglages.profil = ui.choix;
    reglages.assistant_fait = true;
    enregistrer();
    ui.assistant = false;
  }

  function choisirAccent(couleur: string) {
    reglages.accent = couleur;
    enregistrer();
  }
</script>

<div class="scene">
  <div class="carte">
    <div class="entete">
      <span class="logo"><Marque taille={34} /></span>
      <div class="identite">
        <div class="nom display">Cairn</div>
        <div class="etape mono">{t('Setup, step {0} of 3', ui.etape)}</div>
      </div>
      <div class="points">
        {#each [1, 2, 3] as n (n)}
          <span
            class="point"
            style="width:{ui.etape === n ? '26px' : '10px'};background:{ui.etape === n
              ? accent
              : ui.etape > n
                ? rgba(accent, 0.5)
                : 'rgba(255,255,255,0.14)'};"
          ></span>
        {/each}
      </div>
    </div>

    <div class="corps step-in">
      {#if ui.etape === 1}
        <div class="titre display">{t('Find the game')}</div>
        <div class="chapo">
          {t(
            'The launcher needs to know where ManiaPlanet lives. It read the path from the game protocol already registered on this computer, so there is usually nothing to type.',
          )}
        </div>

        {#if saisieChemin}
          <!-- La saisie manuelle. Elle existait dans la maquette et elle
               reste indispensable : sans elle, un joueur dont le registre
               ne porte pas le jeu n'a aucun recours. -->
          <div class="saisie">
            <div class="etiquette">{t('Path to ManiaPlanet.exe')}</div>
            <!-- svelte-ignore a11y_autofocus -->
            <input
              class="txt mono champ"
              type="text"
              bind:value={brouillon}
              autofocus
              onkeydown={(e) => e.key === 'Enter' && validerChemin()}
              aria-label={t('Path to ManiaPlanet.exe')}
            />
            <div class="etiquette haut">{t('Common locations')}</div>
            <div class="suggestions">
              {#each CHEMINS_COURANTS as endroit (endroit.path)}
                <button
                  class="pick-row suggestion"
                  type="button"
                  onclick={() => (brouillon = endroit.path)}
                  style="background:{brouillon === endroit.path
                    ? 'rgba(255,255,255,0.05)'
                    : 'transparent'};border-color:{brouillon === endroit.path
                    ? accent
                    : 'rgba(255,255,255,0.09)'};"
                >
                  <span class="dossier"><Icone nom="folder" taille={15} /></span>
                  <span class="suggestion-texte">
                    <span class="suggestion-nom">{t(endroit.label)}</span>
                    <span class="chemin mono">{endroit.path}</span>
                  </span>
                </button>
              {/each}
            </div>
          </div>
        {:else if systeme.cheminJeu}
          <div class="trouve">
            <span class="coche" style="background:{rgba(accent, 0.16)};color:{accent};">
              <Icone nom="check" taille={17} epaisseur={2.2} />
            </span>
            <div class="trouve-texte">
              <div class="trouve-titre">{t('Game found')}</div>
              <div class="chemin mono">{systeme.cheminJeu}</div>
            </div>
          </div>
          <div class="changer">
            <Bouton
              libelle={t('Change location')}
              genre="fantome"
              {accent}
              onclick={ouvrirSaisie}
            />
          </div>
        {:else}
          <!-- Le registre n'a rien donne et aucun emplacement de repli ne
               porte le jeu. On montre ou Cairn a cherche plutot qu'un
               message d'echec sec. -->
          <div class="trouve manque">
            <span class="coche" style="background:rgba(255,107,74,0.16);color:#FF6B4A;">
              <Icone nom="info" taille={17} epaisseur={2} />
            </span>
            <div class="trouve-texte">
              <div class="trouve-titre">{t('Game not found')}</div>
              <div class="chapo petit">
                {t('Cairn looked in the places ManiaPlanet usually installs itself:')}
              </div>
              {#each CHEMINS_COURANTS as endroit (endroit.path)}
                <div class="chemin mono">{endroit.path}</div>
              {/each}
            </div>
          </div>
          <div class="changer">
            <Bouton
              libelle={t('Set the path myself')}
              genre="fantome"
              {accent}
              onclick={ouvrirSaisie}
            />
          </div>
        {/if}

        {#if !saisieChemin}
          <div class="note">
            <span class="note-icone"><Icone nom="info" taille={14} epaisseur={1.7} /></span>
            <span>
              {t(
                'The launcher never asks for your ManiaPlanet password. It starts the game with a profile name and a server address, nothing else.',
              )}
            </span>
          </div>
        {/if}
      {:else if ui.etape === 2}
        <div class="titre display">{t('Choose your profile')}</div>
        <div class="chapo">
          {t(
            'These are the profiles ManiaPlanet already keeps on this computer. No password is involved, the launcher only tells the game which one to start.',
          )}
        </div>
        <div class="liste">
          {#each systeme.profils as profil (profil.id)}
            <LigneProfil
              {profil}
              choisi={ui.choix === profil.id}
              {accent}
              onclick={() => (ui.choix = profil.id)}
            />
          {/each}
          {#if systeme.profils.length === 0}
            <div class="chapo petit">
              {t('No profile found on this computer')}
            </div>
          {/if}
        </div>
      {:else}
        <div class="titre display">{t('Pick an accent color')}</div>
        <div class="chapo">
          {t(
            'Only a starting point. Backgrounds, palettes and saved themes all live in the Appearance tab, behind the gear icon.',
          )}
        </div>
        <div class="teintes">
          {#each ACCENTS as teinte (teinte.color)}
            <button
              class="swatch"
              type="button"
              title={teinte.name}
              aria-label={teinte.name}
              onclick={() => choisirAccent(teinte.color)}
              style="background:{teinte.color};box-shadow:{accent.toLowerCase() ===
              teinte.color.toLowerCase()
                ? `0 0 0 3px #13161B, 0 0 0 5px ${teinte.color}`
                : 'none'};"
            ></button>
          {/each}
          <input
            class="colorin"
            type="color"
            value={accent}
            title={t('Custom color')}
            aria-label={t('Custom color')}
            oninput={(e) => choisirAccent(e.currentTarget.value)}
          />
        </div>
        <div class="note encadre">
          <span style="color:{accent};display:flex;">
            <Icone nom="drop" taille={16} epaisseur={1.7} />
          </span>
          <span>
            {t('The whole launcher is already following this color, look at the light behind this card.')}
          </span>
        </div>
      {/if}
    </div>

    <div class="onb-foot pied">
      <!-- Le cale garde l'action principale a droite quand il n'y a pas
           de retour possible. C'est un vrai bouton rendu invisible plutot
           qu'un element vide : un element vide est moins haut, et la
           carte perdait deux pixels a la premiere etape. -->
      {#if saisieChemin}
        <Bouton
          libelle={t('Cancel')}
          genre="fantome"
          {accent}
          onclick={() => (saisieChemin = false)}
        />
        <Bouton libelle={t('Use this path')} {accent} onclick={validerChemin} />
      {:else}
        <span class:invisible={ui.etape === 1}>
          <Bouton
            libelle={t('Back')}
            genre="fantome"
            {accent}
            onclick={() => ui.etape > 1 && (ui.etape -= 1)}
          />
        </span>
        <Bouton
          libelle={ui.etape === 3 ? t('Finish setup') : t('Continue')}
          {accent}
          onclick={suivant}
        />
      {/if}
    </div>
  </div>
</div>

<style>
  .scene {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 30px;
    position: relative;
    z-index: 2;
  }
  .carte {
    width: 100%;
    max-width: 530px;
    background: #13161b;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 18px;
    padding: 30px 32px 26px;
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
  }
  .entete {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    display: flex;
    flex-shrink: 0;
  }
  .identite {
    flex: 1;
  }
  .nom {
    font-weight: 600;
    font-size: 13.5px;
  }
  .etape {
    font-size: 11px;
    color: #838c96;
    margin-top: 2px;
  }
  .points {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .point {
    height: 4px;
    border-radius: 999px;
    display: inline-block;
    transition: width 0.2s ease;
  }
  .corps {
    margin-top: 26px;
    height: 336px;
  }
  .titre {
    font-weight: 700;
    font-size: 22px;
    letter-spacing: -0.02em;
  }
  .chapo {
    font-size: 13px;
    color: #9aa3ad;
    line-height: 1.6;
    margin-top: 9px;
  }
  .petit {
    font-size: 11.5px;
  }
  .trouve {
    margin-top: 22px;
    display: flex;
    align-items: center;
    gap: 14px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 15px 17px;
  }
  .manque {
    align-items: flex-start;
  }
  .coche {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .trouve-texte {
    min-width: 0;
    flex: 1;
  }
  .trouve-titre {
    font-size: 12.5px;
    font-weight: 600;
  }
  .chemin {
    font-size: 11.5px;
    color: #9aa3ad;
    margin-top: 3px;
    word-break: break-all;
  }
  .note {
    margin-top: 15px;
    display: flex;
    gap: 10px;
    align-items: flex-start;
    font-size: 11.5px;
    color: #838c96;
    line-height: 1.6;
  }
  .note-icone {
    color: #838c96;
    display: flex;
    flex-shrink: 0;
    margin-top: 1px;
  }
  .encadre {
    margin-top: 18px;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 13px 16px;
    font-size: 12px;
    color: #9aa3ad;
  }
  .saisie {
    margin-top: 22px;
  }
  .etiquette {
    font-size: 12px;
    color: #838c96;
    margin-bottom: 8px;
  }
  .haut {
    font-size: 11.5px;
    margin: 16px 0 8px;
  }
  .champ {
    width: 100%;
    padding: 11px 13px;
    font-size: 12px;
  }
  .suggestions {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .suggestion {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 10px 13px;
    border-radius: 10px;
    cursor: pointer;
    text-align: left;
    color: #f4f6f8;
    border: 1px solid rgba(255, 255, 255, 0.09);
  }
  .dossier {
    color: #838c96;
    display: flex;
    flex-shrink: 0;
  }
  .suggestion-texte {
    flex: 1;
    min-width: 0;
  }
  .suggestion-nom {
    display: block;
    font-size: 12px;
    font-weight: 600;
  }
  .suggestion .chemin {
    font-size: 10.5px;
    margin-top: 2px;
  }
  .changer {
    margin-top: 13px;
  }
  .liste {
    margin-top: 22px;
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  .teintes {
    margin-top: 22px;
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .swatch {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    padding: 0;
  }
  .colorin {
    width: 40px;
    height: 36px;
  }
  .pied {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 26px;
  }
  .invisible {
    visibility: hidden;
  }
</style>
