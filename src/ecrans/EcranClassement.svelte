<!--
  L'ecran Classement.

  **Les donnees sont inventees pour l'instant**, voir `lib/factice.ts`.
  Elles viendront du hub a l'etape 35, et de lui seul : un resultat de
  match est rapporte par le serveur dedie, jamais par le client, qui
  pourrait affirmer n'importe quoi puisque Cairn est ouvert.

  **La tete d'ecran avant la table, et c'est l'essentiel.** Flow decrit
  cet ecran comme celui ou l'on vient « se faire mousser ». Une table
  seule fait regarder les autres ; ce qu'on veut, c'est que le joueur se
  voie d'abord, sache ce qui lui manque pour monter, et regarde ensuite
  qui il doit rattraper. D'ou son rang en grand, sa jauge de palier, et
  l'ecart en points avec la place au dessus.
-->
<script lang="ts">
  import Palier from '../composants/Palier.svelte';
  import Icone from '../lib/Icone.svelte';
  import { t, langueCourante } from '../lib/traduction.svelte';
  import { PALIERS, POINTS_PAR_PALIER } from '../lib/constantes';
  import { CLASSEMENT, MOI } from '../lib/factice';

  /** Ce qui separe le joueur de la place juste au dessus. */
  const aRattraper = $derived.by(() => {
    const devant = CLASSEMENT.find((l) => l.place === MOI.place - 1);
    if (!devant) return null;
    const ecart =
      devant.palier * POINTS_PAR_PALIER +
      devant.points -
      (MOI.palier * POINTS_PAR_PALIER + MOI.points);
    return { joueur: devant.joueur, ecart };
  });

  const ratio = (v: number, d: number) => (v + d === 0 ? 0 : Math.round((v / (v + d)) * 100));

  /**
   * Le taux de victoires, ecrit comme on l'ecrit dans la langue lue.
   *
   * Le francais pose une espace insecable devant le signe pourcent,
   * l'anglais n'en met aucune. Ecrire « 56 % » dans les deux revient a
   * se tromper une fois sur deux.
   */
  const pourcent = (v: number, d: number) =>
    langueCourante() === 'fr' ? `${ratio(v, d)}\u00a0%` : `${ratio(v, d)}%`;
</script>

<div class="ecran">
  <header class="pad entete">
    <div>
      <h1 class="titre display">{t('Leaderboard')}</h1>
      <p class="compte">{t('{0} ranked players', CLASSEMENT.length)}</p>
    </div>
  </header>

  <div class="pad">
    <!-- La carte du joueur. Elle porte la couleur de son palier plutot
         que l'accent de l'application : c'est son rang qu'elle annonce,
         pas l'interface. -->
    <div class="moi" style="--teinte:{PALIERS[MOI.palier].couleur};">
      <div class="place display">#{MOI.place}</div>
      <div class="qui">
        <div class="pseudo display">{MOI.joueur}</div>
        <Palier palier={MOI.palier} points={MOI.points} />
      </div>
      <div class="chiffres">
        <div class="bloc">
          <span class="valeur display">{MOI.points}</span>
          <span class="cle">{t('points')}</span>
        </div>
        <div class="bloc">
          <span class="valeur display">{pourcent(MOI.victoires, MOI.defaites)}</span>
          <span class="cle">{t('{0} W / {1} L', MOI.victoires, MOI.defaites)}</span>
        </div>
      </div>
      {#if aRattraper}
        <div class="ecart">
          <Icone nom="up" taille={15} couleur={PALIERS[MOI.palier].couleur} epaisseur={2} />
          <span>{t('{0} points behind {1}', aRattraper.ecart, aRattraper.joueur)}</span>
        </div>
      {/if}
    </div>
  </div>

  <div class="pad liste">
    <div class="ligne entetes">
      <span>#</span>
      <span>{t('Player')}</span>
      <span>{t('Rank')}</span>
      <span class="chiffre">{t('Points')}</span>
      <span class="chiffre">{t('Win rate')}</span>
      <span class="chiffre">{t('Matches')}</span>
    </div>
    {#each CLASSEMENT as l (l.joueur)}
      <div class="ligne" class:soi={l.soi} style="--teinte:{PALIERS[l.palier].couleur};">
        <span class="place mono">{l.place}</span>
        <span class="pseudo">{l.joueur}</span>
        <span class="rang"><Palier palier={l.palier} points={l.points} etire /></span>
        <span class="chiffre mono">{l.points}</span>
        <span class="chiffre mono">{pourcent(l.victoires, l.defaites)}</span>
        <span class="chiffre mono">{l.victoires + l.defaites}</span>
      </div>
    {/each}
  </div>

  <p class="pad note">
    <Icone nom="info" taille={15} couleur="#838C96" epaisseur={1.7} />
    <span>{t('Sample data. Real standings arrive with the match server.')}</span>
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
    margin-bottom: 14px;
  }
  .titre {
    font-size: 21px;
    font-weight: 600;
    margin: 0;
  }
  .compte {
    font-size: 12px;
    color: #838c96;
    margin: 3px 0 0;
  }

  /* La carte du joueur */
  .moi {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 20px;
    padding: 18px 22px;
    border-radius: 14px;
    background: linear-gradient(
      100deg,
      color-mix(in srgb, var(--teinte) 13%, #13161b) 0%,
      #13161b 62%
    );
    border: 1px solid color-mix(in srgb, var(--teinte) 34%, transparent);
    margin-bottom: var(--air-section, 40px);
  }
  .moi .place {
    font-size: 30px;
    font-weight: 700;
    color: var(--teinte);
    line-height: 1;
  }
  .qui {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  .pseudo {
    font-size: 16px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chiffres {
    display: flex;
    gap: 26px;
  }
  .bloc {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 3px;
  }
  .valeur {
    font-size: 19px;
    font-weight: 600;
  }
  .cle {
    font-size: 11px;
    color: #838c96;
    white-space: nowrap;
  }
  /* L'ecart avec la place du dessus passe sous la carte : c'est une
     information de relance, pas un chiffre d'etat. */
  .ecart {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #9aa3ad;
    padding-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    margin-top: 4px;
  }

  /* La table. Colonnes fixes, comme la liste des serveurs : on balaye
     une colonne, on ne relit pas une structure a chaque ligne. */
  .liste {
    display: flex;
    flex-direction: column;
  }
  .ligne {
    display: grid;
    /* Le pseudo est plafonne et la colonne Rang prend le reste. Sans ce
       plafond, le pseudo s'etirait sur sept cents pixels et les chiffres
       partaient si loin du nom qu'il fallait suivre la ligne du doigt. */
    grid-template-columns: 34px minmax(110px, 200px) minmax(120px, 1fr) 76px 76px 76px;
    align-items: center;
    gap: 12px;
    padding: 0 10px;
    height: 38px;
    border-radius: 8px;
    position: relative;
  }
  .entetes {
    height: 28px;
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #6f7883;
  }
  .liste .ligne:not(.entetes):nth-child(even) {
    background: rgba(255, 255, 255, 0.018);
  }
  .liste .ligne:not(.entetes):hover {
    background: rgba(255, 255, 255, 0.05);
  }
  /* Sa propre ligne, reperable sans etre criarde : un filet a la couleur
     du palier, comme le filet ambre des favoris dans la liste des
     serveurs. Meme vocabulaire, donc rien de neuf a apprendre. */
  .ligne.soi {
    background: color-mix(in srgb, var(--teinte) 9%, transparent) !important;
  }
  .ligne.soi::before {
    content: '';
    position: absolute;
    left: 0;
    top: 5px;
    bottom: 5px;
    width: 3px;
    border-radius: 3px;
    background: var(--teinte);
  }
  .ligne .place {
    color: #6f7883;
    font-size: 12px;
    text-align: right;
  }
  .ligne .pseudo {
    font-size: 13px;
    font-weight: 500;
  }
  .ligne.soi .pseudo {
    font-weight: 600;
  }
  .rang {
    display: flex;
    min-width: 0;
    padding-right: 14px;
  }
  .chiffre {
    text-align: right;
    font-size: 12px;
    color: #9aa3ad;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: var(--air-section, 40px);
    font-size: 11.5px;
    color: #838c96;
  }
  @media (max-width: 900px) {
    .ligne {
      grid-template-columns: 34px minmax(100px, 180px) minmax(90px, 1fr) 76px 76px;
    }
    .ligne > :last-child {
      display: none;
    }
  }
</style>
