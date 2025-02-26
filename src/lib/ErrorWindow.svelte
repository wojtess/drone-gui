<script>
import { emit, listen } from '@tauri-apps/api/event'

let errors = []

for (let index = 0; index < 50; index++) {
    errors = [...errors, "asd"]
}

listen('error', (event) => {
    // errors = [...errors, event.payload]
})
</script>

<div>
    {#each errors as err}
        <p>{err}</p>
    {/each}
</div>

<style>
    p {
        margin-top: -0.0em;
        margin-bottom: -0.0em;
    }
</style>