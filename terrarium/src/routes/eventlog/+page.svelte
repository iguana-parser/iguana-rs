<script lang="ts">
  import { onMount } from "svelte";
  import { emit } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { createMaximizeToggle } from "$lib/window-utils";
  import { commands, type EventLogEntry } from "../../bindings";

  const toggleMaximize = createMaximizeToggle();

  function startDrag(e: MouseEvent) {
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      getCurrentWindow().close();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      emit("debug-step-back");
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      emit("debug-step-forward");
    }
  }

  let eventLog = $state<EventLogEntry[]>([]);
  let currentStep = $state<number>(0);
  let container: HTMLDivElement;

  function scrollToCurrentStep() {
    if (!container) return;
    const currentEl = container.querySelector(`[data-step-index="${currentStep}"]`);
    if (currentEl) {
      currentEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }

  async function handleEntryClick(stepIndex: number | null) {
    if (stepIndex !== null) {
      // Step to the clicked entry
      const result = await commands.debugStepTo(stepIndex);
      if (result.status === "ok") {
        currentStep = result.data.current_step;
        emit("debug-step-changed");
      }
    }
  }

  async function fetchData() {
    // Fetch event log from backend
    const logResult = await commands.getEventLog();
    if (logResult.status === "ok") {
      eventLog = logResult.data;
    }

    // Fetch current step from backend
    const infoResult = await commands.getDebugInfo();
    if (infoResult.status === "ok") {
      currentStep = infoResult.data.current_step;
      setTimeout(() => scrollToCurrentStep(), 50);
    }
  }

  onMount(() => {
    // Fetch data directly from backend
    fetchData();

    // Poll for step changes every 200ms
    const pollInterval = setInterval(async () => {
      const result = await commands.getDebugInfo();
      if (result.status === "ok" && result.data.current_step !== currentStep) {
        currentStep = result.data.current_step;
        setTimeout(() => scrollToCurrentStep(), 50);
      }
    }, 200);

    return () => {
      clearInterval(pollInterval);
    };
  });
</script>

<svelte:window onkeydown={handleKeyDown} />

<svelte:head>
  <title>Event Log - Terrarium</title>
</svelte:head>

<div class="eventlog-window">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="title-bar" onmousedown={startDrag} ondblclick={toggleMaximize}>
    <div class="title-bar-left"></div>
    <div class="title-bar-center">
      <span class="title">Event Log</span>
    </div>
    <div class="title-bar-right"></div>
  </div>
  <div class="eventlog-content" bind:this={container}>
    {#if eventLog.length > 0}
      {#each eventLog as entry}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="event-entry event-{entry.event_type}"
          class:is-step={entry.step_index !== null}
          class:is-current={entry.step_index === currentStep}
          data-step-index={entry.step_index}
          onclick={() => handleEntryClick(entry.step_index)}
        >
          <span class="event-message">{entry.message}</span>
        </div>
      {/each}
    {:else}
      <div class="placeholder">No events loaded</div>
    {/if}
  </div>
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .eventlog-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1e1e1e;
    overflow: hidden;
  }

  .title-bar {
    display: flex;
    align-items: center;
    height: 38px;
    background: #1e1e1e;
    user-select: none;
    flex-shrink: 0;
  }

  .title-bar-left {
    width: 70px;
    flex-shrink: 0;
  }

  .title-bar-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .title {
    font-size: 12px;
    color: #888;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }

  .title-bar-right {
    width: 70px;
    flex-shrink: 0;
  }

  .eventlog-content {
    flex: 1;
    overflow-y: auto;
    font-family: "Fira Code", "Consolas", monospace;
    font-size: 11px;
    padding: 4px 0;
  }

  .event-entry {
    padding: 2px 8px;
    color: #808080;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .event-entry.is-step {
    cursor: pointer;
    color: #d4d4d4;
  }

  .event-entry.is-step:hover {
    background: #2a2d2e;
  }

  .event-entry.is-current {
    background: #264f78;
    color: #fff;
  }

  .event-entry.is-current:hover {
    background: #264f78;
  }

  /* Event type colors */
  .event-processing { color: #4ec9b0; }
  .event-processing.is-current { color: #7fffaa; }

  .event-descriptor { color: #808080; }

  .event-layout { color: #ce9178; }
  .event-layout.is-current { color: #ffb07a; }

  .event-matching { color: #dcdcaa; }
  .event-matching.is-current { color: #fff; }

  .event-match_success { color: #6a9955; }
  .event-match_success.is-current { color: #98c379; }

  .event-match_failed { color: #f14c4c; }
  .event-match_failed.is-current { color: #ff7a7a; }

  .event-gss { color: #569cd6; }
  .event-gss.is-current { color: #7eb8ff; }

  .event-sppf { color: #c586c0; }
  .event-sppf.is-current { color: #e0a0dc; }

  .event-pop { color: #dcdcaa; }
  .event-pop.is-current { color: #fff; }

  .event-call { color: #9cdcfe; }
  .event-call.is-current { color: #c5e4ff; }

  .event-success { color: #6a9955; font-weight: 600; }
  .event-success.is-current { color: #98c379; }

  .event-failed { color: #f14c4c; font-weight: 600; }
  .event-failed.is-current { color: #ff7a7a; }

  .placeholder {
    color: #666;
    font-style: italic;
    padding: 16px;
    text-align: center;
  }
</style>
