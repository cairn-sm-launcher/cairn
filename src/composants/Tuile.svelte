<!--
  Une tuile de l'ecran Jouer.

  Quatre couches de fond dans un ordre qui compte, pose au lot 7 et
  inchange depuis :

    1. la planche vectorielle, qui respire au survol
    2. le voile diagonal, qui garde le texte lisible
    3. la teinte, en mix-blend-mode color, qui colore sans couvrir
    4. le glacis, qui garde le balayage visible sur les zones sombres

  La tuile porte isolation:isolate, sans quoi la teinte irait chercher le
  fond de la page en dessous d'elle.

  Ce que le passage a Svelte apporte ici : la planche n'est plus une
  chaine de 40 Ko reinjectee a chaque rendu. Ses noeuds ne bougent plus,
  donc ses animations ne repartent jamais de zero, et le jeton __T0__ que
  la maquette trimballait pour les recaler n'a plus de raison d'etre.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Favorites from '../illustrations/Favorites.svelte';
  import Popular from '../illustrations/Popular.svelte';
  import Browse from '../illustrations/Browse.svelte';
  import { rgba } from '../lib/couleurs';
  import { t } from '../lib/traduction.svelte';
  import NomJeu from './NomJeu.svelte';
  import type { Tuile } from '../lib/constantes';
  import type { SegmentAffiche } from '../lib/pont';

  let {
    tuile,
    accent,
    legende,
    legendeSegments,
    legendeNue,
    onclick,
    aide,
    supplement,
  }: {
    tuile: Tuile;
    accent: string;
    legende: string;
    /**
     * Un nom venu du jeu, pose devant la legende et rendu avec ses
     * couleurs. La tuile Serveur populaire s'en sert : le reste de sa
     * legende, le mode et le nombre de joueurs, est du texte a nous.
     */
    legendeSegments?: SegmentAffiche[];
    /** Le meme nom, sans ses codes. Voir `NomJeu`. */
    legendeNue?: string;
    onclick: () => void;
    /** L'action du petit point d'interrogation, quand la tuile en a un. */
    aide?: { titre: string; onclick: () => void };
    supplement?: import('svelte').Snippet;
  } = $props();

  // Les trois planches de l'ecran Jouer. Celles d'Editer restent a
  // dessiner, et la tuile sait se passer de planche en attendant.
  const PLANCHES: Record<string, typeof Favorites | undefined> = {
    favorites: Favorites,
    popular: Popular,
    browse: Browse,
  };
  const Planche = $derived(PLANCHES[tuile.id]);
  const couleur = $derived(tuile.couleur ?? accent);
</script>

<div
  class="tile art"
  class:sans-planche={!Planche}
  role="button"
  tabindex="0"
  {onclick}
  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onclick())}
>
  <!-- Les tuiles d'Editer n'ont pas encore de planche : elles gardent les
       trois autres couches, donc la teinte et le balayage au survol, sur
       un fond uni. Une tuile sans dessin doit rester une tuile de la meme
       famille, pas un rectangle etranger pose au milieu des autres. -->
  {#if Planche}
    <div class="tile-photo tile-vec"><Planche /></div>
  {/if}
  <div class="tile-scrim"></div>
  <div class="tile-tint" style="background:{couleur};"></div>
  <div class="tile-glaze" style="background:{rgba(couleur, 0.1)};"></div>

  <div class="tile-body">
    <div class="haut">
      <Icone nom={tuile.icone} taille={21} couleur={tuile.couleur ?? accent} />
      {#if aide}
        <button
          class="helpdot"
          type="button"
          title={aide.titre}
          aria-label={aide.titre}
          onclick={(e) => {
            e.stopPropagation();
            aide.onclick();
          }}
        >
          ?
        </button>
      {/if}
    </div>
    <div class="libelle display">{t(tuile.label)}</div>
    <div class="legende">
      {#if legendeNue}<NomJeu segments={legendeSegments} nu={legendeNue} />{/if}{legende}
    </div>
    {@render supplement?.()}
  </div>
</div>

<style>
  /* Une tuile sans planche n'a pas besoin de la hauteur calibree pour
     les illustrations : sans dessin a montrer, ce qui reste est du vide.
     La hauteur suit quand meme la densite, elle en prend une part, sinon
     les tuiles d'Editer seraient les seules a ne pas respirer avec le
     reste de l'interface. */
  .tile.sans-planche {
    min-height: calc(var(--tuile-hauteur, 178px) * 0.6);
  }
  .haut {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }
  /* Le voile diagonal est dense sur le tiers gauche et s'efface a 84 % de
     la largeur. Le texte doit donc rester dans la moitie gauche, sinon il
     va se promener sur la partie claire de l'illustration.

     La maquette n'avait pas le probleme : ses donnees d'exemple tenaient
     en une ligne courte. Un vrai nom de serveur, lui, ressemble a
     « Channel: SIEGE EVERYTHING ALLOWED 1 ». */
  .libelle,
  .legende {
    max-width: 62%;
  }
  .libelle {
    font-weight: 600;
    font-size: 15px;
    margin-top: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .legende {
    font-size: 12px;
    color: #9aa3ad;
    margin-top: 5px;
    line-height: 1.5;
    /* Deux lignes, puis des points de suspension. La fiche detaillee du
       serveur est a un clic, l'essentiel ici est de reconnaitre le nom. */
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .helpdot {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.06);
    color: #9aa3ad;
    font-size: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }
</style>
