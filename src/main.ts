import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

const cible = document.getElementById('app');
if (!cible) throw new Error("Le point de montage #app est introuvable");

export default mount(App, { target: cible });
