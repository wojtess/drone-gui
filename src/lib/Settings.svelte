<script>
import { createEventDispatcher } from 'svelte';
import Slider from './Slider.svelte'
import { onMount } from 'svelte';
import { invoke } from '@tauri-apps/api/tauri'

let devices = [];

onMount(async () => {
    devices = await invoke('get_devices');
});

const dispatch = createEventDispatcher();

export let sensetivity = 1;
export let device;

if(device == undefined) {
    device = {
        name: ""
    }
}

</script>

<div class="main">
    <div class="left">
        Settings
        <div>
            <Slider min={0.1} max={30} initialValue={sensetivity} bind:value={sensetivity} /> <p>sensetivity</p>
        </div>
        <div>
            <form>
                <select value={device.name} on:change={(e) => {
                    device = devices.find(o => o.name == e.target.value);
                }}>
                    {#each devices as d}
                        <option value={d.name}>
                            {d.name}
                        </option>
                    {/each}
                </select>
            </form>
            <p>network interface</p>
        </div>
    </div>
    <div class="down">
        <div>
            <button on:click={() => {
                dispatch('close', {
                    sensetivity: sensetivity,
                    device: device
                });
            }}>Close</button>
        </div>
    </div>
</div>

<style>

.left > div > p {
    display: inline;
    margin: 0;
}

.left > div {
    display: grid;
    grid-template-columns: auto max-content;
    grid-template-rows: 1fr;
    grid-column-gap: 0px;
    grid-row-gap: 0px; 
}

.left > div:first-child {
    margin-top: 10px;
}

.main{
    border: solid rgb(91, 189, 219);

    display: grid;
    grid-template-columns: auto min-content;
    grid-template-rows: 1fr;
    grid-column-gap: 50px;
    grid-row-gap: 0px;

    padding-right: 5px;
    padding-top: 5px;
    padding-bottom: 5px;
    height: 100%;
}

.down {
    display: flex;
    flex-direction: column
}
.down > div {
    margin-top: auto;
}
</style>