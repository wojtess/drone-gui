<script>
import { createEventDispatcher } from 'svelte';
import { onMount } from 'svelte';
import ErrorWindow from './ErrorWindow.svelte';

const dispatch = createEventDispatcher();

export let sensetivity = 1;

var controlling = false;

var lastMousePos = {
    x:0,
    y:0
};

var controls = {
    pitch:0,
    roll:0,
    yaw:0,
    throttle:0
}

function mouseDownOnCenter(event){
    if(event.button == 2) {
        controlling = true;
        lastMousePos.x = event.clientX;
        lastMousePos.y = event.clientY;
    }
}

function mouseUpOnCenter(event) {
    if(event.button == 2) {
        controlling = false;
    }
}

function mouseMovedOnCenter(event) {
    if(controlling) {
        var dx = event.clientX - lastMousePos.x;
        var dy = event.clientY - lastMousePos.y;
        lastMousePos.x = event.clientX;
        lastMousePos.y = event.clientY;
        controls.pitch += dx * sensetivity / 10;
        controls.roll -= dy * sensetivity / 10;

        if(controls.pitch >= 90) {
            controls.pitch = 90;
        } else if(controls.pitch <= -90) {
            controls.pitch = -90;
        }
        if(controls.roll > 90) {
            controls.roll = 90;
        } else if(controls.roll < -90) {
            controls.roll = -90;
        }
        dispatch("controlsChange", controls);
    }
}

function mouseScrollerOnCenter(event) {
    if(event.deltaY < 0) {
        controls.throttle+=5;
    } else if(event.deltaY > 0) {
        controls.throttle-=5;
    }

    if(controls.throttle > 100) {
        controls.throttle = 100;
    }
    if(controls.throttle < 0) {
        controls.throttle = 0;
    }
    dispatch("controlsChange", controls);
}


</script>

<!-- <button on:click={() => (dispatch('screenChange', 'main'))}>change</button> -->

<div class="main">
    
    <div class="drone">
        <p class="header">drone data</p>
        <div class="data">
            <p>throttle: {controls.throttle}%</p>
            <p>pitch: {controls.pitch.toFixed(2)}°</p>
            <p>roll: {controls.roll.toFixed(2)}°</p>
            <p>yaw: 0.00°</p>
        </div>
    </div>
    <div class="center" on:mousedown={mouseDownOnCenter} on:mouseup={mouseUpOnCenter} on:contextmenu|preventDefault on:mousemove|preventDefault={mouseMovedOnCenter} on:mousewheel={mouseScrollerOnCenter}>
        <center><h1 style="color:#9b9c9a;">VIDEO NOT LOADED</h1></center>
    </div>
    <div class="communication">
        <div class="header">
            <p>communication</p>
        </div>
        <div class="data">
            <p>rrsi: -20db</p>
            <p>last packet sent: 10ms</p>
            <p>last response: 10ms</p>
        </div>

    </div>  
    <div class="error-message">
        <div class="header">
            <p>Errors</p>
        </div>
        <div class="data">
            <ErrorWindow></ErrorWindow>
        </div>
    </div>
</div>


<style>

html, body {
    height: 100%;
    margin: 0;
    padding: 0;
}

.drone > .data {
    padding-top: 8px;
    padding-left: 5px;
}
.communication p {
    margin-top: -0.0em;
    margin-bottom: -0.0em;
}

.header {
    border: rgb(80, 221, 167) .2px;
    border-style: none none solid none;
    text-align: center;
}

.center {
    display: flex;
    justify-content: center;
}

.main {
    height: 100%;
    display: grid;
    grid-template-areas:
        "leftSide body rightTop"
        "leftSide body rightBottom";
    grid-template-rows: 1fr 1fr;
    grid-template-columns: 250px auto 250px;
    gap: 0px;
    }

.center {
    border: solid rgb(80, 221, 167) 2px;
    margin: 2px;

    grid-area: body;
}

.communication {
    border: solid rgb(80, 221, 167) 2px;
    margin: 2px;

    grid-area: rightTop;
}

.drone p {
    margin-top: -0.0em;
    margin-bottom: -0.0em;
}

.drone {
    text-align: left;
    border: solid rgb(80, 221, 167) 2px;
    margin: 2px;

    grid-area: leftSide;
}
.error-message p {
    margin-top: -0.0em;
    margin-bottom: -0.0em;
}
.error-message {
    height: calc(100% - 4px);
    grid-area: rightBottom;
    border: solid rgb(80, 221, 167) 2px;
    margin: 2px;
    overflow: auto;
    box-sizing: border-box;
}

.error-message .data {
    max-height: 200px;
    overflow-y: auto;
}

/* Style for scrollbar in WebKit browsers */
.error-message .data::-webkit-scrollbar {
    width: 12px;
}

.error-message .data::-webkit-scrollbar-track {
    background: #f1f1f1;
}

.error-message .data::-webkit-scrollbar-thumb {
    background: #888;
    border-radius: 6px;
}

.error-message .data::-webkit-scrollbar-thumb:hover {
    background: #555;
}

/* Style for scrollbar in Firefox */
.error-message .data {
    scrollbar-width: thin;
    scrollbar-color: #888 #f1f1f1;
}
</style>
