<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
  import { fly } from 'svelte/transition';
  import { cubicInOut } from 'svelte/easing';

  import type { Timer } from "../types/types";
  import 'overlayscrollbars/overlayscrollbars.css';

  let {
    setStatus,
  }: {
    setStatus: (msg: string) => void;
  } = $props();

  const timerStateKey = 'runningTimer';

  let isEditing = $state<boolean>(false);
  let timerMessage = $state<string | null>('');
  let remainingSeconds = $state<number>(0);
  let initialSeconds = $state<number>(0);
  let displayMinutes = $state<number>(0);
  let displaySeconds = $state<number>(0);
  let isRunning = $state<boolean>(false);
  let interval: ReturnType<typeof setTimeout> | null = null;
  let endAt: number | null = null;
  let isTimerFinished = $state<boolean>(false);

  let editingMinutes = $state<number>(0);
  let editingSeconds = $state<number>(0);
  let editingMessage = $state<string | null>('');

  let selectedInputType = $state<'minutes' | 'seconds' | null>(null);
  let minutesInput = $state<HTMLInputElement>();
  let secondsInput = $state<HTMLInputElement>();

  onMount(async () => {
    await loadTimerFromDB();
    restoreRunningTimer();
  });

  async function loadTimerFromDB() {
    try {
      const timer: Timer = await invoke('get_timer');

      initialSeconds = timer.duration;
      remainingSeconds = timer.duration;
      timerMessage = timer.message;

      setStatus("Fetched timer successfully");

    } catch (error) {
      console.error("Failed to load timer:", error);
      setStatus(`Failed to load timer: ${error}`);

      initialSeconds = 0;
      remainingSeconds = 0;
      timerMessage = '';
    }
  }

  function restoreRunningTimer() {
    const stored = localStorage.getItem(timerStateKey);
    if (!stored) return;

    let { endAt, isRunning: wasRunning, savedInterval } = JSON.parse(stored);
    if (!wasRunning || !endAt) return;

    if (savedInterval) {
      clearInterval(savedInterval);
      savedInterval = null;
    }

    const secondsLeft = Math.ceil((endAt - Date.now()) / 1000);

    if (secondsLeft <= 0) {
      stopTimer();
      return;
    }

    remainingSeconds = secondsLeft;
    isRunning = true;
    interval = setInterval(tickDown, 1000);
  }

  $effect(() => {
    displayMinutes = Math.floor(remainingSeconds / 60)
    displaySeconds = remainingSeconds % 60
  });

  async function setTimer() {
    const totalSeconds = (editingMinutes * 60) + editingSeconds;
    try {
      const timer: Timer = await invoke('create_timer', { duration: totalSeconds, message: editingMessage });

      initialSeconds = timer.duration;
      remainingSeconds = timer.duration;
      timerMessage = timer.message;
      isEditing = false;

      setStatus("Timer set successfully");

    } catch (error) {
      console.error("Failed to set timer:", error);
      setStatus(`Failed to set timer: ${error}`);
    }
  }

  function startEdit() {
    isEditing = true;
    editingMinutes = Math.floor(initialSeconds / 60);
    editingSeconds = initialSeconds % 60;
    editingMessage = timerMessage;
  }

  function cancelEdit() {
    isEditing = false;
    selectedInputType = null;
  }

  function startTimer() {
    if (isRunning || remainingSeconds <= 0) return;

    endAt = Date.now() + remainingSeconds * 1000;
    isRunning = true;
    interval = setInterval(tickDown, 1000);

    localStorage.setItem(
      timerStateKey,
      JSON.stringify({ endAt, isRunning: true, savedInterval: interval, setMessage: timerMessage })
    );
  }

  function tickDown() {
    const stored = localStorage.getItem(timerStateKey);
    if (!stored) return;

    const { endAt } = JSON.parse(stored);
    const secondsLeft = Math.ceil((endAt - Date.now()) / 1000);

    remainingSeconds = secondsLeft;

    if (secondsLeft <= 0) {
      stopTimer();
      isTimerFinished = true;
    }
  }

  function stopTimer() {
    if (interval) {
      clearInterval(interval);
      interval = null;
    }
    isRunning = false;
    localStorage.removeItem(timerStateKey);
  }

  function resetTimer() {
    stopTimer();
    remainingSeconds = initialSeconds;
  }

  function setSelectedInput(type: 'minutes' | 'seconds') {
    selectedInputType = type;
  }

  function increase() {
    if (selectedInputType === 'minutes') {
      editingMinutes++;
    } else if (selectedInputType === 'seconds') {
      editingSeconds = Math.min(editingSeconds + 1, 59);
    }
  }

  function decrease() {
    if (selectedInputType === 'minutes') {
      editingMinutes = Math.max(editingMinutes - 1, 0);
    } else if (selectedInputType === 'seconds') {
      editingSeconds = Math.max(editingSeconds - 1, 0);
    }
  }

</script>

{#if isTimerFinished}
  <div id="timerFinishedContainer" transition:fly={{ x: 100, duration: 400, easing: cubicInOut }}>
      <div id="timerNotificationContainer">
        <OverlayScrollbarsComponent options={{ scrollbars: {autoHide: 'move' as const, autoHideDelay: 800, theme: 'os-theme-dark'}, overflow: { x: "hidden" } }}>
          <div id="timerMessageContainer">
            <p>{timerMessage || 'Timer complete!'}</p>
          </div>
        </OverlayScrollbarsComponent>
        <div class="timerViewSpacer"></div>
        <button class="timerButton" onclick={() => isTimerFinished = false}>OK</button>
      </div>
  </div>
{/if}
<div id="timerContainer">
  <div id="timer">
    <div id="timerCircle">
      <div id="controlsContainer">
        {#if isEditing}
          <div class="timerControls">
            <button class="timerButton" onclick={setTimer}>Save</button>
            <button class="timerButton" onclick={increase}>
              <img id="timeUp-icon" src="up-arrow.svg" alt="upArrow">
            </button>
            <button class="timerButton" onclick={decrease}>
              <img id="timeDown-icon" src="down-arrow.svg" alt="downArrow">
            </button>
            <button class="timerButton" onclick={cancelEdit}>Cancel</button>
          </div>
        {:else}
          <div class="timerControls">
            <button class="timerButton" onclick={startEdit}>Edit</button>
            <button class="timerButton" onclick={startTimer} disabled={isRunning || remainingSeconds <= 0}>Start</button>
            <button class="timerButton" onclick={stopTimer} disabled={!isRunning}>Stop</button>
            <button class="timerButton" onclick={resetTimer}>Reset</button>
          </div>
        {/if}
      </div>
      <div id="contentArea">
        <div id="timeValues">
          {#if isEditing}
            <input type="number" bind:value={editingMinutes} bind:this={minutesInput} min="0" onclick={() => setSelectedInput('minutes')} />
            <input type="number" bind:value={editingSeconds} bind:this={secondsInput} min="0" max="59" onclick={() => setSelectedInput('seconds')} />
          {:else}
            <p id="timeMinutes">{displayMinutes.toString().padStart(2, '0')}</p>
            <p id="timeSeconds">{displaySeconds.toString().padStart(2, '0')}</p>
          {/if}
        </div>
        <div id="notificationMessageBox" role="textbox" tabindex="0" ondblclick={startEdit}>
          <div id="messageContainer">
            {#if isEditing}
              <textarea bind:value={editingMessage} placeholder="Set notification message here"></textarea>
            {:else}
              {#if timerMessage!.length == 0}
                <p class="message">No notification message set</p>
              {:else}
                <p class="message">{timerMessage}</p>
              {/if}
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #timerContainer {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  min-height: 0;
  border: 1px solid #444;
  border-radius: 18px 0 0 0;
  border-bottom: none;
  border-right: none;
}

.timerViewSpacer {
  display: flex;
  flex: 1;
  border-bottom: 1px solid #444;
}

#timer {
  display: flex;
  flex-direction: row;
  flex: 1 1 0;
  min-height: 0;
  gap: 10px;
  width: calc(100vw - 85px);
  padding: 10px;
  overflow-y: auto;
  overflow-x: hidden;
  justify-content: center;
}

#timerCircle {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  align-self: center;
  max-height: calc(100vh - 200px);
  max-width: calc(100vh - 200px);
  height: 100%;
  width: 100%;
  border: 1px solid #444;
  border-radius: 50%;
  padding: calc((100vh - 113px) / 7);
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.3s, box-shadow 0.3s;
}

#timerCircle:hover {
  transform: translateY(-6px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

#contentArea {
  display: flex;
  flex-direction: column;
  justify-content: center;
  flex: 1 1 0;
  padding: 20px;
  gap: 20px;
}

#controlsContainer {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  align-self: center;
  justify-content: center;
  max-height: 50px;
  max-width: 300px;
}

.timerControls {
  display: flex;
  align-self: center;
  flex-direction: row;
  gap: 5px;
}

.timerButton {
  background-color: #222;
  color: #f6f6f6;
  font-size: 16px;
  border: none;
  border-radius: 6px;
  max-width: 80px;
  width: 100%;
  height: 32px;
  padding: 2px 10px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
}

.timerButton:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.timerButton:not(:disabled):hover {
  cursor: pointer;
  background: #333;
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

#timeUp-icon, #timeDown-icon {
  display: flex;
  height: 24px;
  width: 24px;
  filter: brightness(0) invert(0.7);
}

#timeValues {
  display: flex;
  flex: 1 1 0;
  flex-direction: row;
  justify-content: center;
  max-height: 390px;
  padding: 20px;
  gap: 20px;
  border-radius: 8px;
  background: #151515;
}

#timeMinutes, #timeSeconds, #timeValues input {
  flex: 1 1 0;
  text-align: center;
  align-content: center;
  max-width: 200px;
  max-height: 350px;
  font-size: 150px;
  background: #222;
  border-radius: 8px;
  color: #f6f6f6;
  margin: 0;
  border: none;
  outline: none;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
}

#timeValues input[type="number"]::-webkit-inner-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

#timeMinutes:hover, #timeSeconds:hover, #timeValues input:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

#notificationMessageBox {
  display: flex;
  flex-direction: row;
  flex: 1 1 0;
  align-self: flex-end;
  align-items: center;
  max-height: 120px;
  height: 100%;
  width: 100%;
  background: #151515;
  border-radius: 8px;
  padding: 15px;
}

#messageContainer {
  display: flex;
  flex-direction: row;
  flex: 1 1 0;
  padding: 10px;
  width: 100%;
  height: 100%;
  background: #222;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.8);
  transition: transform 0.2s, box-shadow 0.2s;
}

#messageContainer:hover {
  cursor: pointer;
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0,0,0,1);
}

#notificationMessageBox .message {
  max-height: 100px;
  height: 100%;
  width: 100%;
  text-align: left;
  line-height: 17px;
  white-space: pre-wrap;
  word-break: break-word;
  word-wrap: break-word;
  overflow-y: auto;
  margin: 0;
}

#notificationMessageBox .message::-webkit-scrollbar {
  display: none;
}

#notificationMessageBox textarea {
  max-height: 80px;
  height: 100%;
  width: 100%;
  color: #f6f6f6;
  background: transparent;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 17px;
  font-weight: 400;
  border: none;
  outline: none;
  padding: 0;
}

#timerFinishedContainer {
  position: fixed;
  display: flex;
  right: 5px;
  bottom: 25px;
  z-index: 10000;
  max-width: 400px;
  width: 100%;
  max-height: 150px;
  height: 100%;
  background: #151515;
  border: 1px solid #444;
  border-radius: 8px;
}

#timerNotificationContainer {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  padding: 10px;
}

#timerMessageContainer {
  padding-right: 10px;
}

#timerNotificationContainer p {
  flex: 1 1 0;
  overflow-y: auto;
  text-align: left;
  white-space: pre-wrap;
  word-break: break-word;
  word-wrap: break-word;
  margin: 0;
}

#timerNotificationContainer button {
  max-width: 50px;
  width: 100%;
  max-height: 26px;
  height: 100%;
  font-size: 14px;
  margin-top: 4px;
}

#timerNotificationContainer button:hover {
  transform: translateY(-2px);
}

:global {
  .os-theme-dark {
    --os-handle-bg: #888;
    --os-handle-bg-hover: #ccc;
    --os-handle-bg-active: #ccc;
    --os-track-bg: #444;
  }
}

</style>
