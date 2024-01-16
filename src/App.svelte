<script>
  import MainScreen from './lib/MainScreen.svelte'
  import Settings from './lib/Settings.svelte'
  import { emit, listen } from '@tauri-apps/api/event'
  import { invoke } from '@tauri-apps/api/tauri'

  let screen = 'main';
  let screenStack = [];

  function setScreen(newScreen) {
    screenStack.push(screen);
    screen = newScreen;
  }

  function restoreScreen() {
    screen = screenStack.pop();
    if(screen == undefined) {
      screen = 'main';
    }
  }

  listen('openSettings', (e) => {
    setScreen('settings');
  })

  let sensetivity = 1;
  let device;

</script>

<main class="container">
    {#if screen=="main"}
      <MainScreen on:screenChange={(s) => {
          screen=String(s.detail)
        }} on:controlsChange={c => {
          invoke('set_controlls', c.detail);
        }} sensetivity={sensetivity}/>
    {/if}
    {#if screen=='settings'}
      <Settings on:close={(s) => {
        sensetivity = s.detail.sensetivity;
        device = s.detail.device;
        invoke("set_device", {
          device: device
        });
        restoreScreen();
      }} sensetivity={sensetivity} device={device} />
    {/if}
</main>
