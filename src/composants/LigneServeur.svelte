<!--
  Une ligne de la liste des serveurs.

  **Pourquoi une ligne et plus une fiche.** Une fiche est un petit
  paragraphe dont la forme change avec son contenu, donc l'oeil doit
  relire la structure a chaque element. Une liste dont chaque ligne a la
  meme grille transforme la recherche en balayage vertical d'une seule
  colonne, ce qui est plusieurs fois plus rapide.

  **L'occupation est une barre avant d'etre un chiffre.** « 7/32 »
  demande un calcul mental, une barre remplie se lit avant l'attention
  consciente. Le chiffre reste, en second.

  **La colonne d'etat, lot 19d.** Le cadenas etait colle au nom, ou il
  bougeait d'une ligne a l'autre au gre de la longueur des noms : pour
  savoir quels serveurs demandent un mot de passe il fallait lire
  quarante noms. Les marqueurs qui disent un ETAT sont maintenant dans
  une colonne fixe a gauche, ou ils s'alignent et se balayent d'un seul
  mouvement de l'oeil. Restent a cote du nom ceux qui disent une NATURE,
  matchmaking et relais : ils sont rares, et ils se lisent en meme temps
  que le nom parce qu'ils le qualifient.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Bouton from './Bouton.svelte';
  import NomJeu from './NomJeu.svelte';
  import { t } from '../lib/traduction.svelte';
  import type { Serveur } from '../lib/pont';

  let {
    serveur,
    accent,
    ouverte,
    vide = false,
    marquee = false,
    favori = false,
    onbasculer,
    onfavori,
    onrejoindre,
  }: {
    serveur: Serveur;
    accent: string;
    ouverte: boolean;
    /** Une ligne du groupe des serveurs deserts : attenuee au repos. */
    vide?: boolean;
    /** Mise en valeur apres un tirage au sort. */
    marquee?: boolean;
    /** En favori : etoile pleine, et jamais attenuee. */
    favori?: boolean;
    onbasculer: () => void;
    onfavori: () => void;
    onrejoindre: (spectate: boolean) => void;
  } = $props();

  /** L'ambre de la tuile Favorites. Un favori se reconnait a sa couleur. */
  const OR = '#FFC857';

  // La barre se remplit sur la capacite annoncee. Un maximum absurde,
  // 255 places pour quatre joueurs, donne une barre presque vide, et
  // c'est la verite : le serveur est presque vide.
  const remplissage = $derived(
    serveur.maximum > 0 ? Math.min(100, (serveur.joueurs / serveur.maximum) * 100) : 0,
  );
  const habite = $derived(serveur.joueurs > 0 || serveur.observateurs > 0);

  const details = $derived(
    [
      [t('Title pack'), serveur.titre],
      [t('Login'), serveur.login],
      [t('Region'), serveur.zone || '—'],
      [t('Spectators'), `${serveur.observateurs}`],
      serveur.ladder
        ? [t('Ladder'), `${serveur.ladder[0].toLocaleString()} – ${serveur.ladder[1].toLocaleString()}`]
        : null,
      serveur.relais ? [t('Relay of'), serveur.relais] : null,
    ].filter((l): l is [string, string] => l !== null),
  );
</script>

<div
  class="bloc"
  class:vide
  class:ouverte
  class:marquee
  class:favori
  style="--accent:{accent};--or:{OR};"
>
  <div class="rangee">
    <!-- L'etoile est un bouton a elle, hors de celui de la ligne : deux
         boutons imbriques ne sont pas du HTML valide, et surtout mettre
         un serveur en favori ne doit pas deplier sa fiche. -->
    <button
      class="etoile"
      type="button"
      onclick={onfavori}
      aria-pressed={favori}
      title={favori ? t('Remove from favorites') : t('Add to favorites')}
      aria-label={favori ? t('Remove from favorites') : t('Add to favorites')}
    >
      <Icone nom="star" taille={15} epaisseur={1.7} plein={favori} />
    </button>

    <button
      class="ligne"
      type="button"
      onclick={onbasculer}
      aria-expanded={ouverte}
    >
      <span class="etat">
        <span
          class="pastille"
          title={habite ? t('Someone is playing') : t('Empty right now')}
          style="background:{habite ? accent : '#3A424C'};"
        ></span>
        <!-- La place du cadenas est tenue meme quand il n'y a pas de
             cadenas : une colonne qui se decale n'est plus une colonne. -->
        <span class="cadenas" class:present={serveur.prive} title={t('Password required')}>
          {#if serveur.prive}
            <Icone nom="lock" taille={12} epaisseur={2} />
          {/if}
        </span>
      </span>

    <span class="nom">
      <!-- Le titre porte le nom nu : une infobulle ne sait pas afficher
           de couleur, et c'est elle qui sert quand le nom est coupe. -->
      <span class="texte" title={serveur.nom}>
        <NomJeu segments={serveur.nom_segments} nu={serveur.nom} />
      </span>
      <!-- Matchmaking et relais qualifient le serveur lui meme, ils se
           lisent donc avec son nom. Le cadenas, lui, est un etat : il a
           rejoint la colonne de gauche au lot 19d. -->
      {#if serveur.lobby}
        <span class="marqueur mot" title={t('Matchmaking lobby')}>{t('Matchmaking')}</span>
      {/if}
      {#if serveur.relais}
        <span class="marqueur mot" title={t('Relay of {0}', serveur.relais)}>{t('Relay')}</span>
      {/if}
    </span>

    <span class="mode">{serveur.mode}</span>

    <span class="occupation">
      <span class="jauge">
        <span
          class="niveau"
          style="width:{remplissage}%;background:{habite ? accent : '#3A424C'};"
        ></span>
      </span>
      <span class="chiffre mono">{serveur.joueurs}/{serveur.maximum}</span>
      <!-- Les observateurs comptent dans le classement mais pas dans la
           jauge. Sans ce rappel, un serveur range parmi ceux ou il y a
           du monde afficherait 0/32 et passerait pour mal classe. -->
      {#if serveur.observateurs > 0}
        <span class="obs" title={t('{0} watching', serveur.observateurs)}>
          <Icone nom="eye" taille={12} epaisseur={1.7} />
          <span class="mono">{serveur.observateurs}</span>
        </span>
      {/if}
    </span>

    <span class="region">{serveur.region}</span>
    <!-- L'API ne donne aucune latence. Plutot qu'un chiffre invente, un
         tiret, qui dit qu'on ne sait pas. -->
    <span class="latence mono">{serveur.latence === null ? '—' : `${serveur.latence} ms`}</span>

      <span class="chevron" class:tourne={ouverte}>
        <Icone nom="chev" taille={15} epaisseur={2} />
      </span>
    </button>
  </div>

  {#if ouverte}
    <div class="detail">
      {#if serveur.description}
        <p class="description">{serveur.description}</p>
      {/if}
      <dl class="champs">
        {#each details as [cle, valeur] (cle)}
          <div class="champ">
            <dt>{cle}</dt>
            <dd class="mono">{valeur}</dd>
          </div>
        {/each}
      </dl>
      {#if serveur.prive}
        <p class="avertissement">
          {t('This server asks for a password. The game will ask for it after Cairn opens it.')}
        </p>
      {:else if serveur.observation_privee}
        <p class="avertissement">{t('Spectating this server asks for a password.')}</p>
      {/if}
      <div class="actions">
        <Bouton libelle={t('Join')} {accent} onclick={() => onrejoindre(false)} />
        <Bouton
          libelle={t('Spectate')}
          genre="fantome"
          {accent}
          onclick={() => onrejoindre(true)}
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .bloc {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }
  .bloc.ouverte {
    background: rgba(255, 255, 255, 0.025);
  }
  /* Le tirage au sort fait defiler jusqu'a sa ligne et la marque : sans
     ca, on est teleporte sans savoir ou. */
  .bloc.marquee {
    box-shadow: inset 2px 0 0 var(--accent, #20e3b2);
  }
  /* Un favori se voit avant tout le reste, c'est la demande du lot 19d :
     etoile pleine et ambree dans la colonne de gauche, filet de la meme
     couleur sur toute la hauteur de la ligne, et jamais d'attenuation
     meme dans le groupe des serveurs vides. Le cadenas, lui, est un
     petit contour gris. Les deux ne se confondent pas une seconde. */
  .bloc.favori {
    box-shadow: inset 2px 0 0 var(--or, #ffc857);
    background: rgba(255, 200, 87, 0.035);
  }
  /* Le tirage passe devant : il est fugace, il doit se voir sur une
     ligne deja favorite. */
  .bloc.favori.marquee {
    box-shadow:
      inset 2px 0 0 var(--accent, #20e3b2),
      inset 0 0 0 1px rgba(255, 200, 87, 0.28);
  }

  /* L'etoile vit hors du bouton de la ligne, d'ou cette grille a deux
     colonnes qui les pose cote a cote. */
  .rangee {
    display: grid;
    grid-template-columns: 30px minmax(0, 1fr);
    align-items: center;
  }
  .etoile {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 30px;
    background: transparent;
    border: 0;
    padding: 0;
    cursor: pointer;
    color: #4b545f;
    transition: color 0.12s ease;
  }
  .etoile:hover {
    color: #9aa3ad;
  }
  .etoile[aria-pressed='true'],
  .favori .etoile:hover {
    color: var(--or, #ffc857);
  }

  .ligne {
    width: 100%;
    display: grid;
    grid-template-columns: 24px minmax(0, 1fr) 104px 178px 84px 58px 22px;
    align-items: center;
    gap: 14px;
    padding: 11px 6px;
    background: transparent;
    border: 0;
    cursor: pointer;
    text-align: left;
    color: #f4f6f8;
    font-size: 13px;
    transition: background 0.12s ease;
  }
  .ligne:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  /* Les serveurs deserts sont attenues au repos et reviennent au survol.
     Ils restent rejoignables, et c'est souvent comme ca qu'une soiree
     commence : les effacer serait mentir sur ce qui est disponible. */
  .vide .ligne {
    opacity: 0.55;
    transition:
      opacity 0.12s ease,
      background 0.12s ease;
  }
  .vide .ligne:hover,
  .vide.ouverte .ligne,
  .vide.favori .ligne {
    opacity: 1;
  }

  /* Pastille et cadenas alignes, chacun a sa place fixe. */
  .etat {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .cadenas {
    display: flex;
    width: 12px;
    color: #838c96;
  }
  .pastille {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    display: inline-block;
  }

  .nom {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }
  .texte {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  }
  .marqueur {
    display: inline-flex;
    align-items: center;
    color: #9aa3ad;
    flex-shrink: 0;
  }
  .mot {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 5px;
    padding: 2px 5px;
    line-height: 1;
  }

  .mode,
  .region {
    color: #9aa3ad;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .occupation {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .jauge {
    flex: 1;
    height: 5px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .niveau {
    display: block;
    height: 100%;
    border-radius: 999px;
    transition: width 0.25s ease;
  }
  .chiffre {
    font-size: 11px;
    color: #9aa3ad;
    white-space: nowrap;
  }

  .obs {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10.5px;
    color: #838c96;
    flex-shrink: 0;
  }

  .latence {
    font-size: 11px;
    color: #838c96;
    text-align: right;
  }

  .chevron {
    display: flex;
    justify-content: center;
    color: #6e7681;
    transition: transform 0.15s ease;
  }
  .tourne {
    transform: rotate(90deg);
  }

  .detail {
    padding: 4px 6px 18px 72px;
  }
  .description {
    font-size: 12.5px;
    color: #9aa3ad;
    line-height: 1.6;
    margin: 0 0 14px;
    max-width: 70ch;
  }
  .champs {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 6px 22px;
    margin: 0;
  }
  .champ {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 11.5px;
    padding: 3px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }
  dt {
    color: #838c96;
  }
  dd {
    margin: 0;
    color: #c9d1d9;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .avertissement {
    font-size: 11.5px;
    color: #ffc857;
    line-height: 1.5;
    margin: 12px 0 0;
  }
  .actions {
    display: flex;
    gap: 9px;
    margin-top: 16px;
  }

  /* En fenetre etroite, la region et la latence partent les premieres :
     ce sont les colonnes qui portent le moins d'information, l'une etant
     souvent vide et l'autre toujours inconnue. */
  @media (max-width: 1100px) {
    .ligne {
      grid-template-columns: 24px minmax(0, 1fr) 96px 162px 22px;
    }
    .region,
    .latence {
      display: none;
    }
  }
  @media (max-width: 860px) {
    .ligne {
      grid-template-columns: 24px minmax(0, 1fr) 150px 22px;
    }
    .mode {
      display: none;
    }
  }
</style>
