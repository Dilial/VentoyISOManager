import { mount } from 'svelte';
import App from './App.svelte';
import "./lib/i18n";
import { waitLocale } from 'svelte-i18n';

let app: any;

waitLocale().then(() => {
  app = mount(App, {
    target: document.getElementById("app")!,
  });
});

export default app;
