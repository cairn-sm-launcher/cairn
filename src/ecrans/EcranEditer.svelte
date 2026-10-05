<!--
  L'ecran Editer : ce que le joueur a sur son disque.

  **Pourquoi cet ecran ne fait pas ce que la maquette promettait.** Elle
  portait « Create a map », « Edit a replay » et « Title tools », trois
  tuiles qui promettaient d'ouvrir un editeur. Verification faite sur la
  documentation officielle au lot 22d, le jeu ne sait pas le faire :
  les drapeaux de ligne de commande sont /profile, /title, /silent,
  /useexedir, /userdir, /testtitle et /checkdataintegrity, et le
  protocole maniaplanet:// connait join, spectate, qjoin, addfavorite,
  campaign, menutitle, openstore et menustations. Aucun n'ouvre
  l'editeur de maps, celui de replays ni les outils de titre.

  Trois tuiles qui ouvrent le jeu et laissent chercher auraient repete
  le defaut que le plan avait identifie pour l'ancien onglet Reglages :
  la place la plus chere de l'application depensee pour un relais poli.

  L'ecran dit donc ce que Cairn sait vraiment faire, et que les menus du
  jeu font mal : montrer ce qu'on a. Combien de maps, quand pour la
  derniere fois, comment ca se repartit, et quels titres sont installes.
  Trois actions reelles derriere : ouvrir le dossier, ouvrir le jeu, et
  pour un title pack ouvrir le jeu **sur son menu**, d'ou l'editeur est
  a portee.

  Rien n'est lu a l'interieur d'un fichier, voir `cairn_core::inventaire`.
-->
<script lang="ts">
  import Tuile from '../composants/Tuile.svelte';
  import GrilleTuiles from '../composants/GrilleTuiles.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Icone from '../lib/Icone.svelte';
  import { t, langueCourante } from '../lib/traduction.svelte';
  import { TUILES_EDITER } from '../lib/constantes';
  import { reglages, ui, systeme, message, chargerInventaire } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';

  const accent = $derived(reglages.accent);

  const VIDE: pont.Inventaire = {
    nombre: 0,
    octets: 0,
    dernier: null,
    groupes: [],
    recents: [],
  };
  const inv = (id: string): pont.Inventaire => systeme.inventaire[id] ?? VIDE;

  /**
   * Une taille en octets, arrondie au dixieme jusqu'au gigaoctet.
   *
   * Les ordres de grandeur vont de quelques kilooctets pour une map
   * vide a plus d'un gigaoctet pour un title pack. Une unite fixe
   * donnerait soit des zeros, soit des nombres a dix chiffres.
   */
  function poids(octets: number): string {
    if (octets < 1024) return t('{0} B', octets);
    const ko = octets / 1024;
    if (ko < 1024) return t('{0} kB', arrondi(ko));
    const mo = ko / 1024;
    if (mo < 1024) return t('{0} MB', arrondi(mo));
    return t('{0} GB', arrondi(mo / 1024));
  }

  /** L'etiquette regionale, pour la virgule decimale et les milliers. */
  const region = () => (langueCourante() === 'fr' ? 'fr-FR' : 'en-US');

  /** Un dixieme jusqu'a cent, l'unite au dela : 1,2 Go mais 427 Mo. */
  function arrondi(valeur: number): string {
    const x = valeur >= 100 ? Math.round(valeur) : Math.round(valeur * 10) / 10;
    return x.toLocaleString(region());
  }

  /** Une date de fichier, dans la langue de l'interface. */
  function quand(secondes: number | null): string {
    if (secondes === null) return '';
    return new Date(secondes * 1000).toLocaleDateString(region(), {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  /** Le nom d'un sous-dossier, ou le mot qui designe la racine. */
  const groupe = (nom: string) => (nom === '' ? t('Root folder') : nom);

  const legendes = $derived({
    maps: legende('maps', t('No map yet')),
    replays: legende('replays', t('No replay yet')),
    packs: legende('packs', t('No title pack installed')),
  });

  function legende(id: string, vide: string): string {
    if (!systeme.inventaireCharge) return t('Loading...');
    const { nombre, octets, dernier } = inv(id);
    if (nombre === 0) return vide;
    const compte =
      id === 'maps'
        ? t('{0} maps', nombre)
        : id === 'replays'
          ? t('{0} replays', nombre)
          : t('{0} titles', nombre);
    // La date du plus recent porte plus que la taille : elle repond a
    // « ou en etais-je », qui est la question qu'on se pose en ouvrant
    // cet ecran. Elle passe donc devant.
    return dernier === null
      ? `${compte}, ${poids(octets)}`
      : `${compte}, ${t('last {0}', quand(dernier))}, ${poids(octets)}`;
  }

  async function ouvrirLeDossier(id: string) {
    try {
      await pont.ouvrirDossier(id as pont.SorteFichiers);
    } catch (e) {
      message(String(e));
    }
  }

  async function ouvrirLeTitre(titre: string) {
    if (!reglages.profil) {
      message(t('Pick a profile first'));
      return;
    }
    message(t('Opening {0}...', titre));
    try {
      await pont.ouvrirTitre(titre, reglages.profil);
    } catch (e) {
      message(String(e));
    }
  }

  // Lue a l'ouverture de l'ecran et pas dans la boucle de veille :
  // parcourir un dossier Downloaded de plusieurs milliers de maps toutes
  // les cinq secondes serait le defaut que le lot 19d a corrige ailleurs.
  $effect(() => {
    if (!systeme.inventaireCharge) chargerInventaire();
  });
</script>

<div class="pad ecran">
  <GrilleTuiles
    grille="editer"
    ordre={reglages.ordre_tuiles_editer}
    tuiles={TUILES_EDITER}
    {accent}
    etirer={false}
  >
    {#snippet contenu(id, tuile, etouffe)}
      <Tuile
        {tuile}
        {accent}
        legende={legendes[id as keyof typeof legendes] ?? ''}
        onclick={() => {
          if (etouffe()) return;
          ui.editerOuvert = ui.editerOuvert === id ? null : id;
        }}
      >
        {#snippet supplement()}
          {#if ui.editerOuvert === id}
            {@const inventaire = inv(id)}
            <div class="fiche slide-in" role="presentation" onclick={(e) => e.stopPropagation()}>
              {#if inventaire.nombre === 0}
                <p class="rien">
                  {systeme.inventaireCharge
                    ? t('Cairn found nothing in this folder.')
                    : t('Loading...')}
                </p>
              {:else}
                {@const repartis = inventaire.groupes.length > 1}
                <div class="colonnes" class:seule={!repartis}>
                  <!-- La colonne des emplacements ne s'affiche que quand
                       elle dit quelque chose. Les title packs sont tous
                       a la racine du meme dossier : « À la racine, 4 »
                       repeterait le compte deja lu sur la tuile, dans
                       une colonne a elle. -->
                  {#if repartis}
                    <div>
                      <div class="entete">{t('Where they are')}</div>
                      {#each inventaire.groupes as g (g.nom)}
                        <div class="ligne">
                          <span class="cle">{groupe(g.nom)}</span>
                          <span class="valeur mono">{g.nombre}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                  <div class="recents">
                    <div class="entete">{t('Most recent')}</div>
                    {#each inventaire.recents as fichier (fichier.groupe + '/' + fichier.nom)}
                      <!-- La date est la pour tous, et le bouton s'ajoute
                           quand il y a un titre a ouvrir. Mettre l'un OU
                           l'autre a la meme place desalignait la colonne
                           d'une ligne a l'autre. -->
                      <div class="ligne">
                        <span class="cle nom" title={fichier.nom}>{fichier.nom}</span>
                        <span class="valeur mono">{quand(fichier.modifie)}</span>
                        {#if fichier.titre}
                          <button
                            class="ouvrir"
                            type="button"
                            title={t('Open {0} in the game', fichier.titre)}
                            aria-label={t('Open {0} in the game', fichier.titre)}
                            onclick={() => ouvrirLeTitre(fichier.titre ?? '')}
                          >
                            <Icone nom="create" taille={13} epaisseur={1.8} />
                          </button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
              <div class="actions">
                <Bouton
                  libelle={t('Open folder')}
                  genre="fantome"
                  {accent}
                  onclick={() => ouvrirLeDossier(id)}
                />
                <Bouton
                  libelle={t('Refresh')}
                  genre="fantome"
                  {accent}
                  onclick={chargerInventaire}
                />
              </div>
            </div>
          {/if}
        {/snippet}
      </Tuile>
    {/snippet}
  </GrilleTuiles>

  <!-- Dire pourquoi il n'y a pas de bouton « ouvrir l'editeur » vaut
       mieux que de laisser chercher ce qui manque. Une explication
       courte, posee une fois, et qui donne le chemin reel. -->
  <div class="note">
    <Icone nom="info" taille={15} couleur="#838C96" epaisseur={1.7} />
    <span>
      {t(
        'ManiaPlanet cannot be opened straight into its editors, so Cairn opens a title pack on its own menu, where the editor sits one click away.',
      )}
    </span>
  </div>
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
  /* Meme fond opaque que la fiche de la tuile Popular, et pour la meme
     raison : elle s'ouvre par dessus la tuile, et un nom de map long se
     poserait sur la partie claire. */
  .fiche {
    margin-top: 12px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(13, 16, 20, 0.86);
    border-radius: 10px;
    padding: 12px 14px;
  }
  .colonnes {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
  }
  /* Deux colonnes des que la tuile est assez large. En dessous, elles
     s'empilent : une colonne de 80 px couperait tous les noms. */
  @media (min-width: 1180px) {
    .colonnes {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr);
    }
    .colonnes.seule {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .entete {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #6f7883;
    margin-bottom: 5px;
  }
  .ligne {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: 11.5px;
    padding: 3px 0;
    min-height: 20px;
  }
  .cle {
    color: #838c96;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .nom {
    color: #c9d1d9;
  }
  .valeur {
    color: #838c96;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .rien {
    margin: 0;
    font-size: 11.5px;
    color: #838c96;
  }
  .ouvrir {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 20px;
    flex-shrink: 0;
    padding: 0;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(255, 255, 255, 0.05);
    color: #9aa3ad;
    cursor: pointer;
  }
  .ouvrir:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #f4f6f8;
  }
  .actions {
    display: flex;
    gap: 9px;
    margin-top: 14px;
  }
  .actions :global(button) {
    flex: 1;
    padding: 9px 12px;
    font-size: 12px;
    border-radius: 9px;
  }
  .note {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    margin-top: var(--air-section, 40px);
    font-size: 11.5px;
    color: #838c96;
    line-height: 1.55;
    max-width: 720px;
  }
  .note :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
  }
</style>
