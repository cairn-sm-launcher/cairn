<!--
  L'apercu de la densite.

  Flow a du faire des allers-retours entre les reglages et l'accueil,
  puis comparer deux captures, pour voir ce que changeait le reglage.
  C'est le symptome le plus net qu'un reglage manque de retour : on le
  pose a l'aveugle, on va verifier ailleurs, on revient.

  Cette boite est une maquette de l'ecran Jouer en petit : la barre du
  bandeau, les trois tuiles, le bloc des actualites. Les ecarts ne sont
  pas recopies, ils sont calcules a partir des **memes variables** que la
  vraie page, simplement multipliees par une echelle. Un apercu qui
  s'ecarterait de la realite ferait plus de mal que pas d'apercu du tout.

  Elle montre la densite survolee tant qu'on survole, la densite retenue
  sinon : on compare les trois sans rien changer, et on ne valide que ce
  qu'on a vu.
-->
<script lang="ts">
  let {
    densite,
    accent,
  }: {
    densite: string;
    accent: string;
  } = $props();
</script>

<div class="apercu" data-densite={densite} style="--accent:{accent};" aria-hidden="true">
  <span class="bandeau"></span>
  <span class="tuiles">
    <span></span><span></span><span></span>
  </span>
  <span class="actus"></span>
</div>

<style>
  .apercu {
    /* Une maquette au quart. L'echelle et la hauteur vont ensemble :
       `aere` est le cas le plus haut, et il doit tenir en entier dans la
       boite, sinon le bloc des actualites sort par le bas et l'apercu
       ment sur le reglage le plus genereux. Le banc d'essai mesure ce
       debordement aux trois densites. */
    --echelle: 0.24;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    height: 166px;
    padding: 11px 12px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.02);
    overflow: hidden;
  }
  .apercu span {
    display: block;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .apercu .bandeau {
    height: calc(var(--bandeau) * var(--echelle));
    background: color-mix(in srgb, var(--accent) 30%, #2a3138);
  }
  /* La specificite compte ici : `.apercu span` pose display:block sur
     tous les enfants, y compris la rangee de tuiles, qui doit rester en
     flex. Les regles ci dessous reprennent donc `.apercu` en tete. */
  .apercu .tuiles {
    display: flex;
    gap: calc(var(--gouttiere) * var(--echelle));
    margin-top: calc(var(--air-haut) * var(--echelle));
  }
  .apercu .tuiles span {
    flex: 1;
    height: calc(var(--tuile-hauteur) * var(--echelle));
    background: #2a3138;
  }
  .apercu .actus {
    height: calc(88px * var(--echelle));
    margin-top: calc(var(--air-section) * var(--echelle));
    background: #20262c;
  }

  /* Le mouvement est l'information : sans lui il faudrait comparer deux
     images de memoire, ce qui est exactement le probleme qu'on repare. */
  .apercu .bandeau,
  .apercu .tuiles,
  .apercu .tuiles span,
  .apercu .actus {
    transition:
      margin-top 0.22s cubic-bezier(0.32, 0, 0.28, 1),
      height 0.22s cubic-bezier(0.32, 0, 0.28, 1),
      gap 0.22s cubic-bezier(0.32, 0, 0.28, 1);
  }
  :global(.no-anim) .apercu .bandeau,
  :global(.no-anim) .apercu .tuiles,
  :global(.no-anim) .apercu .tuiles span,
  :global(.no-anim) .apercu .actus {
    transition: none;
  }
</style>
