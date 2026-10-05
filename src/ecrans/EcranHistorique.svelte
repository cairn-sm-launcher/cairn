<!--
  L'ecran Historique.

  **Les donnees sont inventees pour l'instant**, voir `lib/factice.ts`,
  comme pour le Classement. Elles viendront du hub a l'etape 35.

  **Les colonnes sont celles de l'annexe de Flow**, dans son ordre :
  resultat, points de rang, score de l'equipe, score adverse, captures /
  lasers / roquettes du joueur, les memes pour l'equipe, la map, la date.

  Deux choix d'ergonomie qui ne vont pas de soi, et qu'il vaut mieux
  ecrire que redecouvrir plus tard :

  1. **Le resultat et les points de rang sont colles a gauche**, avant les
     scores. C'est la question qu'on se pose en ouvrant l'ecran, « est ce
     que je monte », pas « combien de roquettes j'ai mis ». Les statistiques
     de detail viennent apres, pour qui les cherche.

  2. **Les triplets C/L/R restent des triplets**, une seule cellule et non
     trois colonnes. Neuf colonnes de chiffres se ressemblent toutes, et
     l'oeil perd sa ligne. Grouper « 2 / 7 / 11 » donne trois reperes au
     lieu de neuf, et la comparaison avec la ligne de l'equipe juste a
     cote se fait sans compter les colonnes.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Segments from '../composants/Segments.svelte';
  import { t, langueCourante } from '../lib/traduction.svelte';
  import { reglages } from '../lib/etat.svelte';
  import { VICTOIRE, DEFAITE } from '../lib/constantes';
  import { PARTIES, type Partie } from '../lib/factice';

  type Filtre = 'tout' | 'victoires' | 'defaites';
  let filtre = $state<Filtre>('tout');

  const accent = $derived(reglages.accent);

  const options = $derived([
    { id: 'tout', libelle: t('All') },
    { id: 'victoires', libelle: t('Wins') },
    { id: 'defaites', libelle: t('Losses') },
  ]);

  const visibles = $derived(
    PARTIES.filter(
      (p) => filtre === 'tout' || (filtre === 'victoires' ? p.gagne : !p.gagne),
    ),
  );

  const victoires = $derived(PARTIES.filter((p) => p.gagne).length);
  const defaites = $derived(PARTIES.length - victoires);

  /** Le cumul des points de rang sur la periode affichee. */
  const bilan = $derived(visibles.reduce((somme, p) => somme + p.delta, 0));

  const region = () => (langueCourante() === 'fr' ? 'fr-FR' : 'en-US');

  /** Une date de partie, dans la langue de l'interface. */
  const quand = (secondes: number) =>
    new Date(secondes * 1000).toLocaleDateString(region(), {
      month: 'short',
      day: 'numeric',
    });

  /** Le signe est toujours ecrit, y compris le plus : une colonne ou
      seuls les negatifs portent un signe se lit moins vite. */
  const signe = (n: number) => (n > 0 ? `+${n}` : `${n}`);

  const triplet = (v: Partie['moi']) => v.join(' / ');
</script>

<div class="ecran" style="--gagne:{VICTOIRE}; --perdu:{DEFAITE};">
  <header class="pad entete">
    <div>
      <h1 class="titre display">{t('History')}</h1>
      <p class="compte">{t('{0} wins, {1} losses', victoires, defaites)}</p>
    </div>
    <div class="droite">
      <span class="bilan display" class:perte={bilan < 0} style="--teinte:{accent};">
        {signe(bilan)}
      </span>
      <Segments
        {options}
        choix={filtre}
        {accent}
        libelle={t('Filter matches')}
        onchoisir={(id) => (filtre = id as Filtre)}
      />
    </div>
  </header>

  <div class="pad">
    <div class="liste">
      <div class="ligne entetes">
        <span>{t('Result')}</span>
        <span class="chiffre">{t('Rank')}</span>
        <span class="chiffre">{t('Score')}</span>
        <span class="chiffre" title={t('Captures, lasers, rockets')}>{t('You')}</span>
        <span class="chiffre" title={t('Captures, lasers, rockets')}>{t('Team')}</span>
        <span>{t('Map')}</span>
        <span class="chiffre">{t('Date')}</span>
      </div>
      {#each visibles as p (p.id)}
        <!-- `data-quand` porte la date brute. Elle ne sert qu'au banc
             d'essai : la date affichee est ecrite dans la langue lue, et
             un controle qui essaie de la relire passerait sans rien voir
             le jour ou l'analyse echoue. -->
        <div class="ligne" class:gagne={p.gagne} data-quand={p.date}>
          <span class="issue">
            <span class="marque display">{p.gagne ? t('W') : t('L')}</span>
          </span>
          <span class="chiffre delta mono">{signe(p.delta)}</span>
          <span class="chiffre score mono">
            <b>{p.score}</b><span class="tiret">-</span>{p.scoreAdverse}
          </span>
          <span class="chiffre stat mono">{triplet(p.moi)}</span>
          <span class="chiffre stat mono faible">{triplet(p.equipe)}</span>
          <span class="map">{p.map}</span>
          <span class="chiffre date">{quand(p.date)}</span>
        </div>
      {/each}
      {#if visibles.length === 0}
        <p class="vide">{t('Nothing to show with this filter.')}</p>
      {/if}
    </div>
  </div>

  <p class="pad note">
    <Icone nom="info" taille={15} couleur="#838C96" epaisseur={1.7} />
    <span>{t('Sample data. Real matches arrive with the match server.')}</span>
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
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 18px;
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
  .droite {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  /* Le cumul de la periode affichee. Il est a l'accent et non a la
     couleur du palier : ce n'est pas un rang, c'est une tendance. */
  .bilan {
    font-size: 19px;
    font-weight: 700;
    color: var(--teinte);
  }
  .bilan.perte {
    color: var(--perdu);
  }

  .liste {
    display: flex;
    flex-direction: column;
    /* La table ne s'etire pas au dela de ce que ses colonnes demandent.
       Le Classement peut prendre toute la largeur parce que sa jauge de
       palier la remplit ; ici la derniere colonne est un nom de map, et
       l'etirer ne fait que reculer la date hors du regard.

       Le plafond est porte par un bloc INTERIEUR au `.pad` et non par le
       `.pad` lui meme : celui-ci se centre par marges automatiques, donc
       lui donner une largeur plus petite le detache de la gouttiere et
       la table ne s'aligne plus sur le titre. C'est arrive. */
    max-width: 1000px;
  }
  .ligne {
    display: grid;
    grid-template-columns: 58px 58px 74px 86px 86px minmax(0, 1fr) 70px;
    align-items: center;
    gap: 12px;
    padding: 0 10px;
    height: 38px;
    border-radius: 8px;
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
  /* Une marque et non une ligne entiere teintee : vingt lignes vertes et
     rouges empilees deviennent illisibles, alors qu'une colonne de
     marques se balaye.

     Elle ne s'appelle pas « pastille » : ce mot designe deja la pastille
     d'etat du bandeau, et deux choses portant le meme nom dans le meme
     projet finissent par etre confondues, dans le code comme dans les
     controles du banc d'essai, qui les a justement melangees. */
  .marque {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 21px;
    height: 21px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 700;
    background: color-mix(in srgb, var(--perdu) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--perdu) 55%, transparent);
    color: var(--perdu);
  }
  .gagne .marque {
    background: color-mix(in srgb, var(--gagne) 14%, transparent);
    border-color: color-mix(in srgb, var(--gagne) 50%, transparent);
    color: var(--gagne);
  }
  .delta {
    color: var(--perdu);
    font-size: 12px;
    font-weight: 600;
  }
  .gagne .delta {
    color: var(--gagne);
  }
  .score {
    font-size: 12.5px;
    color: #9aa3ad;
  }
  .score b {
    color: #e6e9ee;
    font-weight: 600;
  }
  .tiret {
    color: #5b636d;
    padding: 0 3px;
  }
  .stat {
    font-size: 12px;
    color: #9aa3ad;
  }
  /* Le triplet de l'equipe est volontairement plus pale : il sert de
     toile de fond a celui du joueur, pas de chiffre a lire d'abord. */
  .faible {
    color: #6f7883;
  }
  .map {
    font-size: 12.5px;
    color: #c2c8d0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .date {
    font-size: 11.5px;
    color: #6f7883;
  }
  .chiffre {
    text-align: right;
  }
  .issue {
    display: flex;
  }
  .vide {
    font-size: 12.5px;
    color: #838c96;
    padding: 22px 10px;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-top: var(--air-section, 40px);
    font-size: 11.5px;
    color: #838c96;
  }
  /* En fenetre etroite, les triplets de l'equipe partent avant la map :
     c'est la colonne la moins consultee des deux. */
  @media (max-width: 980px) {
    .ligne {
      grid-template-columns: 58px 58px 74px 86px minmax(0, 1fr) 70px;
    }
    .ligne > :nth-child(5) {
      display: none;
    }
  }
</style>
