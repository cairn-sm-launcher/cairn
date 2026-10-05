<!--
  Les réglages du launcher, derrière la roue dentée.
  **Le seul endroit de Cairn qui porte le mot Réglages.**

  Deux sous-onglets, Général et Apparence, comme la maquette. La section
  Jeu vit dans Général, où elle est descendue au lot 19c-1 bis quand
  l'onglet principal du même nom a été rendu à Communauté.

  Communauté et Développeur s'ajouteront aux lots 25 et 23.
-->
<script lang="ts">
  import Icone from '../lib/Icone.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Interrupteur from '../composants/Interrupteur.svelte';
  import Segments from '../composants/Segments.svelte';
  import Rangee from '../composants/Rangee.svelte';
  import ApercuDensite from '../composants/ApercuDensite.svelte';
  import { t } from '../lib/traduction.svelte';
  import { etiquette } from '../lib/clavier.svelte';
  import { ACCENTS, PALETTES, TUILES, TUILES_EDITER } from '../lib/constantes';
  import {
    reglages,
    ui,
    systeme,
    message,
    enregistrer,
    enregistrerMaintenant,
    rangerTuiles,
    changerLangue,
    naviguer,
  } from '../lib/etat.svelte';
  import * as pont from '../lib/pont';

  const accent = $derived(reglages.accent);

  /**
   * La densite survolee dans le groupe de choix, s'il y en a une.
   *
   * L'apercu montre celle la tant qu'on survole, et la densite retenue
   * sinon. On compare donc les trois sans rien changer, et on ne valide
   * que ce qu'on a deja vu.
   */
  let densiteSurvolee = $state<string | null>(null);
  const densiteMontree = $derived(densiteSurvolee ?? reglages.densite);
  let sousOnglet = $state<'general' | 'apparence'>('general');
  let capture = $state<'precedent' | 'suivant' | null>(null);
  let confirmerRaz = $state(false);

  function garder() {
    enregistrer();
  }

  // ------------------------------------------------------------------
  //  Les raccourcis
  // ------------------------------------------------------------------

  /**
   * Capture la prochaine touche et la retient par sa **position**.
   *
   * `event.code` et non `event.key` : la touche étiquetée A chez Flow
   * porte Q chez un anglophone, et c'est la même position. Le contrôle
   * de ce qui est acceptable est en Rust, où il est testé, donc on
   * enregistre et on relit ce que Rust a bien voulu garder.
   */
  function auClavierCapture(e: KeyboardEvent) {
    if (!capture) return;
    e.preventDefault();
    if (e.code === 'Escape') {
      arreterCapture();
      return;
    }
    const autre =
      capture === 'precedent' ? reglages.touche_suivant : reglages.touche_precedent;
    if (e.code === autre) {
      message(t('That key is already used by the other tab shortcut.'));
      arreterCapture();
      return;
    }

    const sens = capture;
    const voulu = e.code;
    const avant = sens === 'precedent' ? reglages.touche_precedent : reglages.touche_suivant;
    if (sens === 'precedent') reglages.touche_precedent = voulu;
    else reglages.touche_suivant = voulu;
    arreterCapture();

    // On enregistre et on regarde ce que Rust a retenu. Il refuse les
    // touches qui font deja quelque chose, Tab, Entree, Echap, et c'est
    // lui qui a raison : la regle est chez lui, testee, et la dupliquer
    // ici serait la condamner a diverger. La commande rend les reglages
    // assainis, donc l'affichage revient tout seul a la verite.
    enregistrerMaintenant()
      .then(() => {
        const retenu =
          sens === 'precedent' ? reglages.touche_precedent : reglages.touche_suivant;
        if (retenu === voulu) return;
        // Refuse. Rust est reparti sur sa valeur par defaut, ce qui est
        // juste de son point de vue mais pas du notre : le joueur avait
        // peut-etre deja son propre raccourci, et une touche refusee ne
        // doit pas lui coûter celui qu'il avait. On repose donc l'ancien.
        message(t('That key cannot be used as a shortcut.'));
        if (retenu === avant) return;
        if (sens === 'precedent') reglages.touche_precedent = avant;
        else reglages.touche_suivant = avant;
        enregistrerMaintenant().catch(() => {});
      })
      .catch(() => {});
  }

  function demarrerCapture(sens: 'precedent' | 'suivant') {
    capture = sens;
    ui.captureTouche = true;
  }

  function arreterCapture() {
    capture = null;
    ui.captureTouche = false;
  }

  // ------------------------------------------------------------------
  //  L'ordre des tuiles
  // ------------------------------------------------------------------

  // Les fleches passent par la meme regle que le glisser-deposer de
  // l'accueil, en Rust. Elles faisaient leur echange elles memes, ce qui
  // marchait, mais laissait deux facons de ranger les tuiles libres de
  // diverger le jour ou l'une des deux changerait.
  function deplacerTuile(grille: pont.Grille, id: string, pas: -1 | 1) {
    rangerTuiles(grille, { id, pas });
  }

  /** Les deux grilles, cote a cote dans la meme section. */
  const GRILLES = [
    { grille: 'jouer' as const, titre: 'Play screen', tuiles: TUILES },
    { grille: 'editer' as const, titre: 'Edit screen', tuiles: TUILES_EDITER },
  ];

  const ordreDe = (grille: pont.Grille) =>
    grille === 'editer' ? reglages.ordre_tuiles_editer : reglages.ordre_tuiles;

  // ------------------------------------------------------------------
  //  L'apparence
  // ------------------------------------------------------------------

  const PALETTES_LISTE = $derived([
    ...Object.entries(PALETTES).map(([id, p]) => ({ id, libelle: p.label, points: p.dots })),
    {
      id: 'custom',
      libelle: t('Custom'),
      points: reglages.fond_personnalise,
    },
  ]);

  function apparenceCourante(): pont.Apparence {
    return {
      accent: reglages.accent,
      fond_palette: reglages.fond_palette,
      fond_intensite: reglages.fond_intensite,
      fond_vitesse: reglages.fond_vitesse,
      fond_personnalise: [...reglages.fond_personnalise] as [string, string, string],
    };
  }

  function enregistrerEmplacement(i: number) {
    const themes = [...reglages.themes];
    themes[i] = { ...themes[i], apparence: apparenceCourante() };
    reglages.themes = themes;
    garder();
    message(t('Saved'));
  }

  function chargerEmplacement(i: number) {
    const a = reglages.themes[i]?.apparence;
    if (!a) return;
    reglages.accent = a.accent;
    reglages.fond_palette = a.fond_palette;
    reglages.fond_intensite = a.fond_intensite;
    reglages.fond_vitesse = a.fond_vitesse;
    reglages.fond_personnalise = [...a.fond_personnalise] as [string, string, string];
    garder();
  }

  function nommerEmplacement(i: number, nom: string) {
    const themes = [...reglages.themes];
    themes[i] = { ...themes[i], nom };
    reglages.themes = themes;
    garder();
  }

  function remettreAZero() {
    reglages.accent = '#20E3B2';
    reglages.fond_palette = 'storm';
    reglages.fond_intensite = 'medium';
    reglages.fond_vitesse = 'slow';
    reglages.fond_personnalise = ['#20E3B2', '#5B8CFF', '#FF6B4A'];
    confirmerRaz = false;
    garder();
  }

  async function ouvrirLesReglagesDuJeu() {
    if (!reglages.profil) {
      message(t('Pick a profile first'));
      return;
    }
    try {
      await pont.ouvrirJeu(reglages.profil);
      message(t('Opening ManiaPlanet settings...'));
    } catch (e) {
      message(String(e));
    }
  }

  function refaireAssistant() {
    reglages.assistant_fait = false;
    garder();
    ui.etape = 1;
    ui.choix = reglages.profil ?? '';
    ui.assistant = true;
  }
</script>

<svelte:window onkeydown={auClavierCapture} />

<div class="pad ecran">
  <div class="colonne">
    <h1 class="titre display">{t('Launcher settings')}</h1>
    <p class="chapo">
      {t('These preferences belong to the launcher. Game settings live in ManiaPlanet.')}
    </p>

    <div class="onglets">
      <Segments
        options={[
          { id: 'general', libelle: t('General') },
          { id: 'apparence', libelle: t('Appearance') },
        ]}
        choix={sousOnglet}
        {accent}
        libelle={t('Settings section')}
        onchoisir={(id) => (sousOnglet = id as 'general' | 'apparence')}
      />
    </div>

    {#if sousOnglet === 'general'}
      <h2 class="section">{t('Navigation')}</h2>
      <div class="bloc">
        <!-- Les raccourcis portent sur la position de la touche. La
             pastille montre l'étiquette réelle de cette position, donc A
             sur un clavier français et Q sur un anglais, sans que
             personne n'ait rien réglé. -->
        <Rangee titre={t('Previous tab')} explication={t('Switches to the tab on the left')}>
          <span class="touche mono" class:vivante={capture === 'precedent'}>
            {capture === 'precedent' ? '…' : etiquette(reglages.touche_precedent)}
          </span>
          <Bouton
            libelle={capture === 'precedent' ? t('Cancel') : t('Change')}
            genre="fantome"
            {accent}
            onclick={() =>
              capture === 'precedent' ? arreterCapture() : demarrerCapture('precedent')}
          />
        </Rangee>
        <Rangee titre={t('Next tab')} explication={t('Switches to the tab on the right')}>
          <span class="touche mono" class:vivante={capture === 'suivant'}>
            {capture === 'suivant' ? '…' : etiquette(reglages.touche_suivant)}
          </span>
          <Bouton
            libelle={capture === 'suivant' ? t('Cancel') : t('Change')}
            genre="fantome"
            {accent}
            onclick={() =>
              capture === 'suivant' ? arreterCapture() : demarrerCapture('suivant')}
          />
        </Rangee>
        <Rangee
          titre={t('Mouse side buttons')}
          explication={t(
            'Back and Forward go to the previous screen, like in a browser. Alt and the arrow keys do the same.',
          )}
          derniere
        >
          <Interrupteur
            actif={reglages.souris_laterale}
            {accent}
            libelle={t('Mouse side buttons')}
            onbasculer={() => {
              reglages.souris_laterale = !reglages.souris_laterale;
              garder();
            }}
          />
        </Rangee>
      </div>
      <p class="note">
        <Icone nom="info" taille={13} epaisseur={1.7} />
        <span>{t('A shortcut is tied to where the key sits, not to the letter on it. It follows you if you change keyboard layout.')}</span>
      </p>

      <h2 class="section">{t('Language')}</h2>
      <div class="bloc">
        <Rangee
          titre={t('Interface language')}
          explication={t('Applies to every screen of the launcher')}
          derniere
        >
          <Segments
            options={[
              { id: 'en', libelle: 'English' },
              { id: 'fr', libelle: 'Français' },
            ]}
            choix={reglages.langue}
            {accent}
            libelle={t('Interface language')}
            onchoisir={(id) => changerLangue(id as 'fr' | 'en')}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Animations')}</h2>
      <div class="bloc">
        <Rangee
          titre={t('Motion')}
          explication={t('Illustrations and the drifting background light. Turn it off for a completely still interface.')}
          derniere
        >
          <Interrupteur
            actif={reglages.animations}
            {accent}
            libelle={t('Motion')}
            onbasculer={() => {
              reglages.animations = !reglages.animations;
              garder();
            }}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Tile layout')}</h2>
      <p class="note">
        <Icone nom="info" taille={14} />
        <span>{t('On the Play and Edit screens you can also drag the tiles, or move a focused one with Ctrl and the arrow keys.')}</span>
      </p>
      <!-- Les deux grilles se reglent au meme endroit. Les separer en deux
           sections aurait donne deux fois la meme rangee de fleches avec
           la meme explication au dessus, ce qui se lit comme un doublon. -->
      {#each GRILLES as g (g.grille)}
        {@const ordre = ordreDe(g.grille)}
        <div class="sous-titre">{t(g.titre)}</div>
        <div class="bloc">
          {#each ordre as id, i (id)}
            <Rangee
              titre={`${i + 1}. ${t(g.tuiles[id]?.label ?? id)}`}
              derniere={i === ordre.length - 1}
            >
              <button
                class="reorder-btn fleche"
                type="button"
                title={t('Move up')}
                aria-label={t('Move up')}
                disabled={i === 0}
                onclick={() => deplacerTuile(g.grille, id, -1)}
              >
                <Icone nom="up" taille={15} epaisseur={2} />
              </button>
              <button
                class="reorder-btn fleche"
                type="button"
                title={t('Move down')}
                aria-label={t('Move down')}
                disabled={i === ordre.length - 1}
                onclick={() => deplacerTuile(g.grille, id, 1)}
              >
                <Icone nom="down" taille={15} epaisseur={2} />
              </button>
            </Rangee>
          {/each}
        </div>
      {/each}

      <h2 class="section">{t('Game')}</h2>
      <div class="bloc">
        <!-- L'ecran Editer est arrive ici a l'etape 30, faute de mieux.
             Le bandeau a cinq onglets de la phase 4 ne lui en laisse
             plus, et un ecran construit au lot 22d ne doit pas devenir
             injoignable pour autant. Emplacement provisoire, a revoir
             avec Flow : c'est le meme raisonnement qui avait fait
             descendre ici l'ancien onglet Reglages du jeu. -->
        <Rangee
          titre={t('My game files')}
          explication={t('Your maps, your replays and the title packs installed on this computer.')}
        >
          <Bouton
            libelle={t('Open')}
            genre="fantome"
            {accent}
            onclick={() => naviguer('edit')}
          />
        </Rangee>
        <Rangee titre={t('Game settings live in ManiaPlanet')} explication={t('To change settings, you need to go in game. Graphics, audio, network and key bindings are all handled by the ManiaPlanet client itself.')}>
          <Bouton libelle={t('Open game settings')} {accent} onclick={ouvrirLesReglagesDuJeu} />
        </Rangee>
        <Rangee titre={t('Game location')}>
          {#snippet children()}
            <span class="chemin mono">{systeme.cheminJeu ?? t('Game not found')}</span>
          {/snippet}
        </Rangee>
        <Rangee
          titre={t('Active profile')}
          explication={t('Started with the game, no password involved')}
          derniere
        >
          <Bouton
            libelle={reglages.profil ?? t('No profile')}
            genre="fantome"
            {accent}
            onclick={() => (ui.selecteurProfil = true)}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Application')}</h2>
      <div class="bloc">
        <Rangee
          titre={t('When you click the window close button')}
          explication={t('Minimizing keeps Cairn in the taskbar. Closing ends it. Whatever you pick, ManiaPlanet keeps running.')}
        >
          <Segments
            options={[
              { id: 'demander', libelle: t('Ask') },
              { id: 'reduire', libelle: t('Minimize') },
              { id: 'quitter', libelle: t('Close') },
            ]}
            choix={reglages.fermeture}
            {accent}
            libelle={t('When you click the window close button')}
            onchoisir={(id) => {
              reglages.fermeture = id as typeof reglages.fermeture;
              garder();
            }}
          />
        </Rangee>
        <Rangee
          titre={t('Start with Windows')}
          explication={t('Cairn opens minimized when you log in. Windows also lists it in its Startup tab, where you can turn it off.')}
          derniere
        >
          <Interrupteur
            actif={reglages.demarrage_auto}
            {accent}
            libelle={t('Start with Windows')}
            onbasculer={() => {
              reglages.demarrage_auto = !reglages.demarrage_auto;
              // Tout de suite et non dans 400 ms : celui-ci ecrit dans le
              // registre, et l'interrupteur doit montrer ce que Rust a
              // vraiment reussi a y mettre.
              enregistrerMaintenant();
            }}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Diagnostics')}</h2>
      <div class="bloc">
        <!-- Pas derriere un interrupteur « mode developpeur ». Cet ecran
             ouvre en lecture ce que Cairn ecrit sur la machine de
             quelqu'un : le cacher irait contre ce qu'il sert a montrer.
             Et c'est le chemin par lequel un testeur envoie un rapport
             lisible plutot qu'un « ca marche pas ». -->
        <Rangee
          titre={t('Log and problem report')}
          explication={t('Read what Cairn writes about itself, and send it with the paths already removed.')}
          derniere
        >
          <Bouton
            libelle={t('Open')}
            genre="fantome"
            {accent}
            onclick={() => naviguer('dev')}
          />
        </Rangee>
      </div>

      <div class="actions">
        <Bouton
          libelle={t('Run first time setup again')}
          genre="fantome"
          {accent}
          onclick={refaireAssistant}
        />
        <Bouton
          libelle={t('Quit launcher')}
          genre="fantome"
          {accent}
          onclick={() => pont.quitter()}
        />
      </div>
    {:else}
      <h2 class="section">{t('Spacing')}</h2>
      <div class="bloc">
        <div class="densite">
          <div class="texte">
            <div class="titre-rangee">{t('Density')}</div>
            <div class="explication">
              {t('How much air there is between the banner, the tiles and the news')}
            </div>
            <Segments
              options={[
                { id: 'compact', libelle: t('Compact') },
                { id: 'equilibre', libelle: t('Balanced') },
                { id: 'aere', libelle: t('Airy') },
              ]}
              choix={reglages.densite}
              {accent}
              libelle={t('Density')}
              onchoisir={(id) => {
                reglages.densite = id as typeof reglages.densite;
                garder();
              }}
              onapercu={(id) => (densiteSurvolee = id)}
            />
          </div>
          <ApercuDensite densite={densiteMontree} {accent} />
        </div>
      </div>

      <h2 class="section">{t('Game text')}</h2>
      <div class="bloc">
        <Rangee
          titre={t('Server name colors')}
          explication={t(
            'Server owners color their names with the game codes. Cairn brightens the ones too dark to read here.',
          )}
          derniere
        >
          <Interrupteur
            actif={reglages.couleurs_jeu}
            {accent}
            libelle={t('Server name colors')}
            onbasculer={() => {
              reglages.couleurs_jeu = !reglages.couleurs_jeu;
              garder();
            }}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Accent color')}</h2>
      <div class="bloc">
        <Rangee
          titre={t('Accent color')}
          explication={t('Used for buttons, the active tab and highlights')}
          derniere
        >
          {#each ACCENTS as teinte (teinte.color)}
            <button
              class="swatch"
              type="button"
              title={teinte.name}
              aria-label={teinte.name}
              onclick={() => {
                reglages.accent = teinte.color;
                garder();
              }}
              style="background:{teinte.color};box-shadow:{accent.toLowerCase() ===
              teinte.color.toLowerCase()
                ? `0 0 0 2px #13161B, 0 0 0 4px ${teinte.color}`
                : 'none'};"
            ></button>
          {/each}
          <input
            class="colorin"
            type="color"
            value={accent}
            title={t('Custom color')}
            aria-label={t('Custom color')}
            oninput={(e) => {
              reglages.accent = e.currentTarget.value;
              garder();
            }}
          />
        </Rangee>
      </div>

      <h2 class="section">{t('Background effect')}</h2>
      <div class="bloc">
        <Rangee titre={t('Intensity')} explication={t('How present the moving light is')}>
          <Segments
            options={[
              { id: 'soft', libelle: t('Soft') },
              { id: 'medium', libelle: t('Medium') },
              { id: 'vivid', libelle: t('Vivid') },
            ]}
            choix={reglages.fond_intensite}
            {accent}
            libelle={t('Intensity')}
            onchoisir={(id) => {
              reglages.fond_intensite = id;
              garder();
            }}
          />
        </Rangee>
        <Rangee
          titre={t('Speed')}
          explication={t('How fast the light drifts across the screen')}
        >
          <Segments
            options={[
              { id: 'slow', libelle: t('Slow') },
              { id: 'normal', libelle: t('Normal') },
              { id: 'fast', libelle: t('Fast') },
            ]}
            choix={reglages.fond_vitesse}
            {accent}
            libelle={t('Speed')}
            onchoisir={(id) => {
              reglages.fond_vitesse = id;
              garder();
            }}
          />
        </Rangee>
        <div class="palettes" class:derniere={reglages.fond_palette !== 'custom'}>
          <div class="titre-rangee">{t('Palette')}</div>
          <div class="explication">{t('Colors used by the drifting light')}</div>
          <div class="choix">
            {#each PALETTES_LISTE as p (p.id)}
              <button
                class="palette"
                type="button"
                aria-pressed={reglages.fond_palette === p.id}
                onclick={() => {
                  reglages.fond_palette = p.id;
                  garder();
                }}
                style="border-color:{reglages.fond_palette === p.id
                  ? accent
                  : 'rgba(255,255,255,0.12)'};background:{reglages.fond_palette === p.id
                  ? 'rgba(255,255,255,0.08)'
                  : 'transparent'};color:{reglages.fond_palette === p.id
                  ? '#F4F6F8'
                  : '#9AA3AD'};"
              >
                <span class="points">
                  {#each p.points as point (point)}
                    <span class="point" style="background:{point};"></span>
                  {/each}
                </span>
                {p.libelle}
              </button>
            {/each}
          </div>
        </div>
        {#if reglages.fond_palette === 'custom'}
          <Rangee
            titre={t('Custom colors')}
            explication={t('Pick the three lights yourself')}
            derniere
          >
            {#each [0, 1, 2] as i (i)}
              <input
                class="colorin"
                type="color"
                value={reglages.fond_personnalise[i]}
                aria-label={t('Custom colors')}
                oninput={(e) => {
                  const trois = [...reglages.fond_personnalise] as [string, string, string];
                  trois[i] = e.currentTarget.value;
                  reglages.fond_personnalise = trois;
                  garder();
                }}
              />
            {/each}
          </Rangee>
        {/if}
      </div>

      <h2 class="section">{t('Saved themes')}</h2>
      <div class="bloc">
        {#each reglages.themes as theme, i (i)}
          <div class="emplacement" class:derniere={i === reglages.themes.length - 1}>
            <span class="rang mono">{i + 1}.</span>
            <input
              class="txt nom"
              type="text"
              value={theme.nom}
              placeholder={theme.apparence ? t('Theme {0}', i + 1) : t('Empty slot')}
              aria-label={t('Theme {0}', i + 1)}
              oninput={(e) => nommerEmplacement(i, e.currentTarget.value)}
            />
            <span class="points">
              {#each theme.apparence ? (theme.apparence.fond_palette === 'custom' ? theme.apparence.fond_personnalise : (PALETTES[theme.apparence.fond_palette]?.dots ?? [])) : ['rgba(255,255,255,0.1)', 'rgba(255,255,255,0.1)', 'rgba(255,255,255,0.1)'] as point, n (n)}
                <span class="point grand" style="background:{point};"></span>
              {/each}
            </span>
            <span class="boutons">
              <Bouton
                libelle={t('Save')}
                genre="fantome"
                {accent}
                onclick={() => enregistrerEmplacement(i)}
              />
              {#if theme.apparence}
                <Bouton
                  libelle={t('Load')}
                  genre="fantome"
                  {accent}
                  onclick={() => chargerEmplacement(i)}
                />
              {/if}
            </span>
          </div>
        {/each}
        <p class="pied-bloc">
          {t('Save stores the current colors and background settings in the slot.')}
        </p>
      </div>

      {#if confirmerRaz}
        <div class="bloc raz">
          <Rangee
            titre={t('Reset the theme?')}
            explication={t('Accent color and background settings go back to default. Saved slots are kept.')}
            derniere
          >
            <Bouton
              libelle={t('Cancel')}
              genre="fantome"
              {accent}
              onclick={() => (confirmerRaz = false)}
            />
            <Bouton libelle={t('Reset')} accent="#FF4D6D" onclick={remettreAZero} />
          </Rangee>
        </div>
      {:else}
        <div class="actions">
          <Bouton
            libelle={t('Reset to default theme')}
            genre="fantome"
            {accent}
            onclick={() => (confirmerRaz = true)}
          />
        </div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .ecran {
    flex: 1;
    overflow-y: auto;
    /* L'ecran Reglages respire lui aussi. C'est ce que Flow attendait
       en changeant la densite : voir quelque chose bouger sous le
       bandeau, la ou on fait le choix. Deux tiers de la valeur de
       l'accueil, parce qu'un ecran de formulaire n'a pas besoin
       d'autant d'air qu'un ecran de tuiles. */
    padding-top: calc(var(--air-haut, 24px) * 0.66 + 6px);
    padding-bottom: 40px;
    position: relative;
    z-index: 1;
  }
  .colonne {
    max-width: 780px;
  }
  .titre {
    font-weight: 700;
    font-size: 19px;
    letter-spacing: -0.02em;
    margin: 0 0 4px;
  }
  .chapo {
    font-size: 12.5px;
    color: #838c96;
    margin: 0 0 18px;
  }
  .onglets {
    margin-bottom: 20px;
  }
  .section {
    font-weight: 600;
    font-size: 12px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #838c96;
    margin: 0 0 11px;
  }
  /* Un cran en dessous du titre de section : les deux grilles sont deux
     sous-parties d'un meme reglage, pas deux reglages. */
  .sous-titre {
    font-size: 11.5px;
    color: #6f7883;
    margin: 0 0 6px 2px;
  }
  .sous-titre:not(:first-of-type) {
    margin-top: 14px;
  }
  .bloc {
    background: #13161b;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 13px;
    margin-bottom: calc(var(--air-section, 28px) * 0.5 + 4px);
    overflow: hidden;
  }
  .raz {
    border-color: rgba(255, 255, 255, 0.14);
  }
  .note {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    font-size: 11.5px;
    color: #838c96;
    line-height: 1.55;
    margin: -10px 2px 18px;
  }
  .actions {
    display: flex;
    gap: 9px;
    flex-wrap: wrap;
    margin-bottom: 18px;
  }

  .touche {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 500;
    font-size: 13px;
    color: #f4f6f8;
  }
  /* Pendant la capture, la case attend une touche et le dit. */
  .vivante {
    border-color: currentColor;
    color: #f4f6f8;
    animation: attente 1.1s ease-in-out infinite;
  }
  @keyframes attente {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.45;
    }
  }

  .fleche {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: transparent;
    color: #9aa3ad;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .fleche:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .chemin {
    font-size: 11.5px;
    color: #9aa3ad;
    word-break: break-all;
    max-width: 420px;
    text-align: right;
  }

  .swatch {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  /* Le choix a gauche, l'apercu a droite, cote a cote : une maquette
     posee sous les boutons obligerait a descendre les yeux, et une
     maquette qu'on ne regarde pas ne sert a rien. En fenetre etroite,
     elle passe dessous. */
  .densite {
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 16px 20px;
  }
  .densite .texte {
    flex: 1 1 auto;
    min-width: 0;
  }
  .densite :global(.apercu) {
    flex: 0 0 236px;
  }
  @media (max-width: 720px) {
    .densite {
      flex-direction: column;
      align-items: stretch;
    }
    .densite :global(.apercu) {
      flex: 1 1 auto;
    }
  }

  .palettes {
    padding: 15px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .palettes.derniere {
    border-bottom: none;
  }
  .titre-rangee {
    font-size: 13.5px;
    font-weight: 600;
  }
  .explication {
    font-size: 12px;
    color: #838c96;
    margin-top: 2px;
    margin-bottom: 13px;
  }
  .choix {
    display: flex;
    gap: 9px;
    flex-wrap: wrap;
  }
  .palette {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 9px 14px;
    border-radius: 10px;
    border: 1px solid;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .points {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
  }
  .point {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    display: inline-block;
  }
  .grand {
    width: 10px;
    height: 10px;
  }

  .emplacement {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 13px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }
  .emplacement.derniere {
    border-bottom: none;
  }
  .rang {
    font-size: 11.5px;
    color: #838c96;
    width: 16px;
    flex-shrink: 0;
  }
  .nom {
    flex: 1;
    min-width: 0;
    max-width: 260px;
  }
  .boutons {
    display: flex;
    gap: 7px;
    margin-left: auto;
    flex-shrink: 0;
  }
  .pied-bloc {
    padding: 12px 20px;
    background: rgba(255, 255, 255, 0.02);
    font-size: 12px;
    color: #838c96;
    margin: 0;
  }

  /* Dans les blocs de reglages les boutons sont plus petits qu'ailleurs :
     ils accompagnent une rangee, ils ne la dominent pas. */
  .bloc :global(button.btn-cta),
  .bloc :global(button.btn-ghost) {
    padding: 8px 14px;
    font-size: 12px;
    border-radius: 9px;
  }
</style>
