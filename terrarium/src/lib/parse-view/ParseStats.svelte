<script lang="ts">
  import type { StatsData } from "../../bindings";
  import "./parse-view.css";

  let {
    statsData,
    instrument,
    parseResultAvailable,
    onCollect,
  }: {
    statsData: StatsData | null;
    instrument: boolean;
    parseResultAvailable: boolean;
    onCollect: () => void;
  } = $props();
</script>

<div class="stats-panel">
  {#if !instrument}
    <div class="stats-empty">Rebuild with the Instrument option enabled to collect stats.</div>
  {:else if !statsData}
    <div class="stats-empty">
      Run a parse to collect stats.
      {#if parseResultAvailable}
        <div style="margin-top: 8px;">
          <button class="parse-btn" onclick={onCollect}>Collect now</button>
        </div>
      {/if}
    </div>
  {:else}
    <div class="stats-counters">
      <div><span class="stats-label">descriptors</span><span class="stats-value">{statsData.descriptors_count}</span></div>
      <div><span class="stats-label">gss_nodes</span><span class="stats-value">{statsData.gss_nodes_count}</span></div>
      <div><span class="stats-label">gss_edges</span><span class="stats-value">{statsData.gss_edges_count}</span></div>
      <div><span class="stats-label">nonterminal_nodes</span><span class="stats-value">{statsData.nonterminal_nodes_count}</span></div>
      <div><span class="stats-label">intermediate_nodes</span><span class="stats-value">{statsData.intermediate_nodes_count}</span></div>
      <div><span class="stats-label">terminal_nodes</span><span class="stats-value">{statsData.terminal_nodes_count}</span></div>
      <div><span class="stats-label">ambiguous_nodes</span><span class="stats-value">{statsData.ambiguous_nodes_count}</span></div>
    </div>
    {#if Object.keys(statsData.histograms).length > 0}
      <div class="stats-histograms">
        <h4>Size histograms</h4>
        {#each Object.entries(statsData.histograms) as [name, lens] (name)}
          {@const lensArr = lens as number[]}
          {@const buckets = (() => {
            const b = [0, 0, 0, 0, 0, 0, 0, 0];
            for (const l of lensArr) {
              if (l === 0) b[0]++;
              else if (l === 1) b[1]++;
              else if (l === 2) b[2]++;
              else if (l <= 4) b[3]++;
              else if (l <= 8) b[4]++;
              else if (l <= 16) b[5]++;
              else if (l <= 32) b[6]++;
              else b[7]++;
            }
            return b;
          })()}
          {@const labels = ['0', '1', '2', '3-4', '5-8', '9-16', '17-32', '33+']}
          {@const max = Math.max(1, ...buckets)}
          {@const n = lensArr.length}
          {@const sum = lensArr.reduce((a: number, b: number) => a + b, 0)}
          {@const maxv = Math.max(0, ...lensArr)}
          <div class="histogram">
            <div class="histogram-name">{name}</div>
            <div class="histogram-meta">n={n}  max={maxv}  avg={(sum / Math.max(1, n)).toFixed(2)}</div>
            {#each buckets as count, i}
              <div class="histogram-row">
                <span class="histogram-bucket">{labels[i]}</span>
                <div class="histogram-bar-container">
                  <div class="histogram-bar" style="width: {(count * 100) / max}%"></div>
                </div>
                <span class="histogram-count">{count}</span>
              </div>
            {/each}
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .stats-panel {
    padding: 16px 20px;
    overflow: auto;
    color: #cccccc;
    font-family: Menlo, monospace;
    font-size: 12px;
    width: 100%;
    height: 100%;
    align-self: stretch;
    box-sizing: border-box;
  }
  .stats-empty { color: #888; padding: 8px 0; }
  .stats-counters > div {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
    border-bottom: 1px solid #2d2d2d;
  }
  .stats-label { color: #888; }
  .stats-value { color: #4ec9b0; font-weight: 600; }
  .stats-histograms { margin-top: 18px; }
  .stats-histograms h4 { margin: 0 0 10px 0; color: #ddd; font-size: 12px; font-weight: 600; }
  .histogram { margin-bottom: 14px; }
  .histogram-name { color: #569cd6; margin-bottom: 2px; }
  .histogram-meta { color: #888; margin-bottom: 4px; font-size: 11px; }
  .histogram-row {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 16px;
  }
  .histogram-bucket {
    width: 38px;
    text-align: right;
    color: #888;
  }
  .histogram-bar-container {
    flex: 1;
    background: #1e1e1e;
    height: 10px;
    border-radius: 2px;
    overflow: hidden;
  }
  .histogram-bar {
    background: #4ec9b0;
    height: 100%;
  }
  .histogram-count {
    width: 36px;
    color: #aaa;
  }
</style>
