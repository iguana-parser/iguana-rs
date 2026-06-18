<script lang="ts">
  import { ChevronDown } from "lucide-svelte";
  import { tick } from "svelte";

  interface Props {
    value: string | null;
    options: string[];
    disabled?: boolean;
    placeholder?: string;
    onchange?: (value: string) => void;
  }

  let {
    value = $bindable(null),
    options,
    disabled = false,
    placeholder = "Select...",
    onchange,
  }: Props = $props();

  let open = $state(false);
  let query = $state("");
  let highlightIndex = $state(0);
  let inputEl: HTMLInputElement | undefined = $state();
  let menuEl: HTMLDivElement | undefined = $state();
  let rootEl: HTMLDivElement | undefined = $state();

  let filtered = $derived(
    query.trim() === ""
      ? options
      : options.filter((o) =>
          o.toLowerCase().includes(query.trim().toLowerCase()),
        ),
  );

  $effect(() => {
    if (highlightIndex >= filtered.length) highlightIndex = 0;
  });

  async function openMenu() {
    if (disabled) return;
    open = true;
    query = "";
    highlightIndex = Math.max(
      0,
      filtered.findIndex((o) => o === value),
    );
    await tick();
    inputEl?.focus();
    scrollHighlightedIntoView();
  }

  function closeMenu() {
    open = false;
    query = "";
  }

  function pick(nt: string) {
    value = nt;
    onchange?.(nt);
    closeMenu();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (filtered.length === 0) return;
      highlightIndex = (highlightIndex + 1) % filtered.length;
      scrollHighlightedIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (filtered.length === 0) return;
      highlightIndex =
        (highlightIndex - 1 + filtered.length) % filtered.length;
      scrollHighlightedIntoView();
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filtered[highlightIndex]) pick(filtered[highlightIndex]);
    } else if (e.key === "Escape") {
      e.preventDefault();
      closeMenu();
    }
  }

  function scrollHighlightedIntoView() {
    if (!menuEl) return;
    const el = menuEl.querySelector<HTMLElement>(
      ".picker-item.highlighted",
    );
    el?.scrollIntoView({ block: "nearest" });
  }

  function onWindowClick(e: MouseEvent) {
    if (!open) return;
    const target = e.target as HTMLElement;
    if (rootEl && !rootEl.contains(target)) {
      closeMenu();
    }
  }
</script>

<svelte:window onclick={onWindowClick} />

<div bind:this={rootEl} class="picker" class:disabled>
  <button
    class="picker-trigger"
    {disabled}
    onclick={() => (open ? closeMenu() : openMenu())}
  >
    <span class="picker-value">{value ?? placeholder}</span>
    <ChevronDown size={14} class="picker-chevron" />
  </button>
  {#if open}
    <div class="picker-menu" bind:this={menuEl}>
      <input
        bind:this={inputEl}
        class="picker-input"
        type="text"
        placeholder="Filter..."
        bind:value={query}
        onkeydown={onKeyDown}
      />
      {#if filtered.length === 0}
        <div class="picker-empty">No matches</div>
      {:else}
        {#each filtered as nt, i}
          <button
            class="picker-item"
            class:highlighted={i === highlightIndex}
            class:selected={nt === value}
            onmouseenter={() => (highlightIndex = i)}
            onclick={() => pick(nt)}
          >
            {nt}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    width: 150px;
  }

  .picker.disabled {
    opacity: 0.5;
  }

  .picker-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 5px 8px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .picker-trigger:hover:not(:disabled) {
    background: #454545;
    border-color: #666;
  }

  .picker-trigger:focus {
    outline: none;
    border-color: #0e639c;
  }

  .picker-trigger:disabled {
    cursor: not-allowed;
  }

  .picker-value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    text-align: left;
  }

  :global(.picker-chevron) {
    flex-shrink: 0;
    color: #888;
  }

  .picker-menu {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 2px;
    background: #2d2d2d;
    border: 1px solid #454545;
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    max-height: 240px;
    overflow-y: auto;
    z-index: 100;
    display: flex;
    flex-direction: column;
  }

  .picker-input {
    margin: 4px;
    padding: 5px 8px;
    background: #3c3c3c;
    color: #d4d4d4;
    border: 1px solid #555;
    border-radius: 3px;
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }

  .picker-input:focus {
    border-color: #0e639c;
  }

  .picker-empty {
    padding: 6px 10px;
    color: #888;
    font-size: 13px;
    font-style: italic;
  }

  .picker-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    color: #d4d4d4;
    border: none;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
  }

  .picker-item.highlighted {
    background: #094771;
  }

  .picker-item.selected {
    background: #0e639c;
  }
</style>
