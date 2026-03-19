<script lang="ts">
  import type { CalendarDay, CalendarEvent } from "../types/types";

  let {
    days,
    events,
    brightColors,
  }: {
    days: CalendarDay[];
    events: CalendarEvent[];
    brightColors: Array<string>;
  } = $props();

  

  const dayNames = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  const hours = Array.from({ length: 24 }, (_, i) => i);
</script>

<div id="weeklyOverlay">
  <div id="mainContent">
    <div id="weekDayAxis">
      {#each dayNames as day (day)}
        <div class="weekDayName">{day}</div>
      {/each}
    </div>
    <div id="eventGrid">
      <div id="hours">
        {#each hours as hour (hour)}
          <div class="hourLabel">{hour.toString().padStart(2, '0')}:00</div>
        {/each}
      </div>
      <div id="timeSlots">

      </div>
    </div>
  </div>
</div>

<style>
  #weeklyOverlay {
    position: absolute;
    top: 70px;
    left: 300px;
    bottom: 0;
    z-index: 10000;
    display: flex;
    width: calc(100vw - 300px);
    background-color: #0f0f0f;
  }

  #mainContent {
    position: relative;
    display: flex;
    flex-direction: column;
    width: calc(100% - 86px);
    padding: 12px;
    padding-left: 0;
    overflow-y: auto;
  }

  #weekDayAxis {
    display: flex;
    flex-direction: row;
    flex: 1 1 0;
    max-height: 30px;
    margin: 0 0 5px 40px;
    gap: 10px;
    user-select: none;
  }

  .weekDayName {
    flex: 1 1 0;
    max-height: 30px;
    align-content: center;
    font-weight: 800;
  }

  #eventGrid {
    display: flex;
    flex: 1 1 0;
  }

  #hours {
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 40px;
    min-width: 40px;
    justify-content: flex-start;
  }

  .hourLabel {
    display: flex;
    flex-direction: column;
    justify-content: center;
    height: 81px;
    font-size: 0.75rem;
  }

  #timeSlots {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-template-rows: repeat(24, 80px);
    flex: 1;
    gap: 1px;
    background: repeating-linear-gradient(to bottom, transparent 0%, transparent 80px, #333 80px, #333 81px);
    background-size: 100%;
    overflow-y: auto;
    border: 1px solid red;
  }
</style>