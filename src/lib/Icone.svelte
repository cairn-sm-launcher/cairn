<!--
  Les icones de l'interface.

  Vingt-quatre traces de 24 par 24, en contour et jamais en aplat, tous
  dessines sur la meme grille. Ils etaient assembles en chaine par une
  fonction svg(), ils sont maintenant un composant : le trace entre par
  {@html}, ce qui est sans risque ici puisqu'il vient de la constante ci
  dessous et de nulle part ailleurs.

  La couleur suit currentColor par defaut, donc une icone posee dans un
  bouton prend la couleur du bouton, y compris au survol.
-->
<script lang="ts">
  const TRACES: Record<string, string> = {
    "eye": "<path d=\"M2.5 12S6 5.8 12 5.8 21.5 12 21.5 12 18 18.2 12 18.2 2.5 12 2.5 12Z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
    "lock": "<rect x=\"5\" y=\"10.5\" width=\"14\" height=\"9.5\" rx=\"2.2\"/><path d=\"M8.2 10.5V7.6a3.8 3.8 0 0 1 7.6 0v2.9\"/>",
    "search": "<circle cx=\"10.8\" cy=\"10.8\" r=\"6.3\"/><path d=\"M15.4 15.4L20.5 20.5\"/>",
    "shuffle": "<path d=\"M17 3.5l3.5 3.5L17 10.5\"/><path d=\"M17 13.5l3.5 3.5L17 20.5\"/><path d=\"M3 7h4l10 10h3.5\"/><path d=\"M3 17h4l2.6-2.6\"/><path d=\"M14.4 9.6L17 7h3.5\"/>",
    "sort": "<path d=\"M4 7h13M4 12h9M4 17h5\"/>",
  "star": "<path d=\"M12 2.5l2.9 6.2 6.8.7-5.1 4.6 1.5 6.7-6.1-3.6-6.1 3.6 1.5-6.7-5.1-4.6 6.8-.7z\"/>",
  "flame": "<path d=\"M12 2s-1 3-3.5 5.5S6 12 6 14.5A6 6 0 0 0 18 14.5c0-2-1-3.5-2-5\"/><path d=\"M12 8c0 2-2 3-2 5a2 2 0 0 0 4 0c0-1-.5-1.5-1-2\"/>",
  "grid": "<rect x=\"4\" y=\"4\" width=\"7\" height=\"7\" rx=\"1.5\"/><rect x=\"13\" y=\"4\" width=\"7\" height=\"7\" rx=\"1.5\"/><rect x=\"4\" y=\"13\" width=\"7\" height=\"7\" rx=\"1.5\"/><rect x=\"13\" y=\"13\" width=\"7\" height=\"7\" rx=\"1.5\"/>",
  "bell": "<path d=\"M12 3a5 5 0 0 0-5 5v2.6c0 .9-.3 1.8-.9 2.5L5 15h14l-1.1-1.9c-.6-.7-.9-1.6-.9-2.5V8a5 5 0 0 0-5-5z\"/><path d=\"M9.5 18a2.5 2.5 0 0 0 5 0\"/>",
  "people": "<circle cx=\"8.5\" cy=\"9\" r=\"3\"/><circle cx=\"16\" cy=\"10\" r=\"2.4\"/><path d=\"M2.5 20c0-3.3 2.7-6 6-6s6 2.7 6 6\"/><path d=\"M14.5 14.3c2.6.4 4.5 2.6 4.5 5.2\"/>",
  "gear": "<circle cx=\"12\" cy=\"12\" r=\"3.2\"/><path d=\"M19.4 13.5a7.6 7.6 0 0 0 0-3l2-1.6-2-3.4-2.4.7a7.6 7.6 0 0 0-2.6-1.5L14 2h-4l-.4 2.7a7.6 7.6 0 0 0-2.6 1.5l-2.4-.7-2 3.4 2 1.6a7.6 7.6 0 0 0 0 3l-2 1.6 2 3.4 2.4-.7a7.6 7.6 0 0 0 2.6 1.5L10 22h4l.4-2.7a7.6 7.6 0 0 0 2.6-1.5l2.4.7 2-3.4z\"/>",
  "back": "<path d=\"M15 18l-6-6 6-6\"/>",
  "home": "<path d=\"M3.5 10.6L12 3.6l8.5 7v8.3a1.5 1.5 0 0 1-1.5 1.5H5a1.5 1.5 0 0 1-1.5-1.5z\"/><path d=\"M9.4 20.4v-6.1h5.2v6.1\"/>",
  "refresh": "<path d=\"M3 12a9 9 0 0 1 15.4-6.4M21 12a9 9 0 0 1-15.4 6.4\"/><path d=\"M18 3v5h-5M6 21v-5h5\"/>",
  "up": "<path d=\"M6 15l6-6 6 6\"/>",
  "down": "<path d=\"M6 9l6 6 6-6\"/>",
  "info": "<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 11.2v5\"/><path d=\"M12 7.6v.6\"/>",
  "chat": "<path d=\"M4 5h16v10H8l-4 4V5z\"/>",
  "create": "<rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"3.5\"/><path d=\"M12 8.5v7M8.5 12h7\"/>",
  "replay": "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"3\"/><path d=\"M10.5 9l5 3-5 3z\"/>",
  "tools": "<path d=\"M15.2 6.1a4 4 0 0 0-5.3 5.3L4 17.3V20h2.7l5.9-5.9a4 4 0 0 0 5.3-5.3l-2.6 2.6-2.1-.5-.5-2.1z\"/>",
  "sliders": "<path d=\"M4 8h8M17 8h3M4 16h3M12 16h8\"/><circle cx=\"14.5\" cy=\"8\" r=\"2.2\"/><circle cx=\"9.5\" cy=\"16\" r=\"2.2\"/>",
  "check": "<path d=\"M20 6.5L9.5 17.5 4.5 12.5\"/>",
  "folder": "<path d=\"M3 7.5A1.5 1.5 0 0 1 4.5 6h4l2 2.5h9A1.5 1.5 0 0 1 21 10v8a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18z\"/>",
  "user": "<circle cx=\"12\" cy=\"8.5\" r=\"3.5\"/><path d=\"M4.5 20c0-3.6 3.4-6.5 7.5-6.5s7.5 2.9 7.5 6.5\"/>",
  "drop": "<path d=\"M12 3.5c3.2 3.6 5.5 6.3 5.5 9a5.5 5.5 0 0 1-11 0c0-2.7 2.3-5.4 5.5-9z\"/>",
  "chev": "<path d=\"M9 6l6 6-6 6\"/>",
  "chevL": "<path d=\"M15 6l-6 6 6 6\"/>",
  "x": "<path d=\"M6 6l12 12M18 6L6 18\"/>",
  };

  let {
    nom,
    taille = 18,
    couleur = 'currentColor',
    epaisseur = 1.6,
    plein = false,
  }: {
    nom: string;
    taille?: number;
    couleur?: string;
    epaisseur?: number;
    /**
     * Remplit le trace au lieu de le laisser en contour.
     *
     * Une seule icone s'en sert, l'etoile des favoris. Un contour et un
     * aplat de la meme forme se distinguent d'un coup d'oeil, a la
     * peripherie du regard et sans lire : c'est ce qu'il faut pour une
     * colonne qu'on balaye. Flow a demande qu'un favori se voie plus
     * qu'un serveur a mot de passe, et c'est la difference qui porte.
     */
    plein?: boolean;
  } = $props();
</script>

<svg
  width={taille}
  height={taille}
  viewBox="0 0 24 24"
  fill={plein ? couleur : 'none'}
  stroke={couleur}
  stroke-width={epaisseur}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {@html TRACES[nom] ?? ''}
</svg>
