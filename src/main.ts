import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";

// Define the keys mapping to pads 0-15
// Top row: 1 2 3 4
// Second row: q w e r
// Third row: a s d f
// Bottom row: z x c v
const keyMap: Record<string, number> = {
  '1': 0, '2': 1, '3': 2, '4': 3,
  'q': 4, 'w': 5, 'e': 6, 'r': 7,
  'a': 8, 's': 9, 'd': 10, 'f': 11,
  'z': 12, 'x': 13, 'c': 14, 'v': 15,
};

let currentTargetPad = 0;

let isBooting = false;

let isBus1Held = false;
let isBus2Held = false;
let currentEditBus = "Bus1";
let isResampleArmed = false;
let isResampleRecording = false;

let typingTimeout: number | undefined;

const typeText = async (element: HTMLElement, text: string, speedMs = 20): Promise<void> => {
  return new Promise(resolve => {
    if (typingTimeout) clearTimeout(typingTimeout);
    element.innerText = "";
    let i = 0;
    
    const typeNext = () => {
      if (i < text.length) {
        element.innerText += text.charAt(i);
        i++;
        typingTimeout = window.setTimeout(typeNext, speedMs);
      } else {
        resolve();
      }
    };
    typeNext();
  });
};

const formatDropMessage = (count: number, startPad: number, endPad: number): string => {
  if (count === 1) {
    return `LOADED 1 IN PAD ${startPad + 1}`;
  }
  return `LOADED ${count} IN PADS ${startPad + 1}-${endPad + 1}`;
};

const playCascadeAnimation = (padIndices: number[]) => {
  padIndices.forEach((padId, index) => {
    setTimeout(() => {
      const padEl = document.querySelector(`[data-pad="${padId}"]`);
      if (padEl) {
        padEl.classList.add("cascade-glow");
        setTimeout(() => padEl.classList.remove("cascade-glow"), 300);
      }
    }, index * 100);
  });
};

window.addEventListener("DOMContentLoaded", () => {
  const pads = document.querySelectorAll<HTMLButtonElement>(".pad");
  const uploadBtn = document.getElementById("audio-upload-btn");
  const targetPadDisplay = document.getElementById("target-pad");
  const statusDisplay = document.getElementById("status-display");
  const resampleBtn = document.getElementById("resample-btn");
  const bus1Btn = document.getElementById("bus1-btn");
  const bus2Btn = document.getElementById("bus2-btn");
  const lcdScreen = document.querySelector(".lcd-screen");

  // Routing Bus Buttons
  bus1Btn?.addEventListener("mousedown", () => { isBus1Held = true; bus1Btn.classList.add("active"); });
  bus1Btn?.addEventListener("touchstart", (e) => { e.preventDefault(); isBus1Held = true; bus1Btn.classList.add("active"); });
  window.addEventListener("mouseup", () => { isBus1Held = false; bus1Btn?.classList.remove("active"); });
  window.addEventListener("touchend", () => { isBus1Held = false; bus1Btn?.classList.remove("active"); });

  bus2Btn?.addEventListener("mousedown", () => { isBus2Held = true; bus2Btn.classList.add("active"); });
  bus2Btn?.addEventListener("touchstart", (e) => { e.preventDefault(); isBus2Held = true; bus2Btn.classList.add("active"); });
  window.addEventListener("mouseup", () => { isBus2Held = false; bus2Btn?.classList.remove("active"); });
  window.addEventListener("touchend", () => { isBus2Held = false; bus2Btn?.classList.remove("active"); });

  // Resample Button
  resampleBtn?.addEventListener("click", async () => {
    if (isBooting) return;
    
    if (isResampleRecording) {
      isResampleRecording = false;
      isResampleArmed = false;
      resampleBtn.classList.remove("resample-recording", "resample-armed");
      lcdScreen?.classList.remove("lcd-resampling");
      if (statusDisplay) typeText(statusDisplay, "READY", 10);
      try { await invoke("set_resampling", { state: false }); } catch (err) { console.error(err); }
      return;
    }

    isResampleArmed = !isResampleArmed;
    if (isResampleArmed) {
      resampleBtn.classList.add("resample-armed");
      lcdScreen?.classList.add("lcd-resampling");
      if (statusDisplay) typeText(statusDisplay, "[RESAMPLING ARMED]", 10);
    } else {
      resampleBtn.classList.remove("resample-armed");
      lcdScreen?.classList.remove("lcd-resampling");
      if (statusDisplay) typeText(statusDisplay, "READY", 10);
    }
  });

  const runBootSequence = async () => {
    isBooting = true;
    if (statusDisplay) statusDisplay.innerText = "";
    await new Promise(r => setTimeout(r, 300));
    if (statusDisplay) await typeText(statusDisplay, "INIT AUDIO...", 40);
    await new Promise(r => setTimeout(r, 600));
    if (statusDisplay) await typeText(statusDisplay, "READY", 40);
    isBooting = false;
  };

  runBootSequence();

  // Function to highlight a pad briefly
  const animatePad = (padId: number) => {
    const padEl = document.querySelector(`[data-pad="${padId}"]`);
    if (padEl) {
      padEl.classList.add("active");
      setTimeout(() => padEl.classList.remove("active"), 150);
    }
  };

  // Trigger Pad function
  const triggerPad = async (padId: number) => {
    if (isBooting) return;
    animatePad(padId);

    if (isBus1Held || isBus2Held) {
      const bus = isBus1Held ? "Bus1" : "Bus2";
      if (statusDisplay) typeText(statusDisplay, `PAD ${padId + 1} -> ${bus.toUpperCase()}`, 10);
      try {
        await invoke("set_pad_bus", { pad: padId, bus });
      } catch (err) {
        console.error("Error setting pad bus:", err);
      }
      return;
    }

    if (isResampleArmed && !isResampleRecording) {
      isResampleArmed = false;
      isResampleRecording = true;
      resampleBtn?.classList.remove("resample-armed");
      resampleBtn?.classList.add("resample-recording");
      if (statusDisplay) typeText(statusDisplay, "[RECORDING]", 10);
      try {
        await invoke("set_resampling", { state: true });
      } catch (err) {
        console.error("Error setting resampling:", err);
      }
    }

    if (statusDisplay && !isResampleRecording) typeText(statusDisplay, `PLAYING PAD ${padId + 1}`, 10);
    try {
      await invoke("trigger_pad", { padId });
    } catch (err) {
      console.error("Error triggering pad:", err);
      if (statusDisplay && !isResampleRecording) typeText(statusDisplay, `ERROR PAD ${padId + 1}`, 10);
    }
  };

  // Set up pad clicks
  pads.forEach(pad => {
    pad.addEventListener("mousedown", () => {
      const padId = parseInt(pad.dataset.pad || "0", 10);
      triggerPad(padId);
    });

    // Right click to set as target pad for loading
    pad.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      if (isBooting) return;
      const padId = parseInt(pad.dataset.pad || "0", 10);
      currentTargetPad = padId;
      if (targetPadDisplay) targetPadDisplay.innerText = (padId + 1).toString();
      
      pads.forEach(p => p.classList.remove("target"));
      pad.classList.add("target");
      setTimeout(() => pad.classList.remove("target"), 300);
      
      if (statusDisplay) typeText(statusDisplay, `TARGET PAD ${padId + 1}`, 10);
    });
  });

  // Handle keyboard
  window.addEventListener("keydown", (e) => {
    if (isBooting) return;
    if (e.repeat) return; // Ignore hold
    const padId = keyMap[e.key.toLowerCase()];
    if (padId !== undefined) {
      triggerPad(padId);
    }
  });

  // Handle file load via Tauri dialog plugin
  uploadBtn?.addEventListener("click", async () => {
    if (isBooting) return;
    try {
      const file = await open({
        multiple: false,
        filters: [{
          name: 'Audio',
          extensions: ['wav', 'mp3']
        }]
      });

      if (!file) return; // User cancelled

      if (statusDisplay) typeText(statusDisplay, "LOADING...", 10);
      
      await invoke("load_audio", { path: file, padId: currentTargetPad });
      if (statusDisplay) typeText(statusDisplay, `LOADED PAD ${currentTargetPad + 1}`, 10);
    } catch (err) {
      console.error("Error loading audio:", err);
      if (statusDisplay) typeText(statusDisplay, "LOAD ERROR", 10);
    }
  });

  // Handle native drag and drop
  getCurrentWebview().onDragDropEvent(async (event) => {
    if (isBooting) return;
    if (event.payload.type === 'over') {
      const { position } = event.payload;
      const el = document.elementFromPoint(position.x, position.y);
      const padEl = el?.closest('.pad');
      pads.forEach(p => p.classList.toggle('drag-hover', p === padEl));
    } else if (event.payload.type === 'leave') {
      pads.forEach(p => p.classList.remove('drag-hover'));
      if (statusDisplay && statusDisplay.innerText === "INVALID FORMAT") {
        statusDisplay.innerText = "";
      }
    } else if (event.payload.type === 'drop') {
      pads.forEach(p => p.classList.remove('drag-hover'));
      const { paths, position } = event.payload;
      const el = document.elementFromPoint(position.x, position.y);
      const padEl = el?.closest('.pad');
      if (!padEl) return;

      const startPadId = parseInt((padEl as HTMLElement).dataset.pad || "0", 10);
      const validExtensions = ['.wav', '.mp3'];
      
      const validPaths = paths.filter(path => {
        const ext = path.substring(path.lastIndexOf('.')).toLowerCase();
        return validExtensions.includes(ext);
      });

      if (validPaths.length !== paths.length) {
        if (statusDisplay) typeText(statusDisplay, "INVALID FORMAT", 10);
        if (validPaths.length === 0) return;
      }

      if (statusDisplay) typeText(statusDisplay, "LOADING...", 10);

      const loadedIndices: number[] = [];

      for (let i = 0; i < validPaths.length; i++) {
        const currentPadId = startPadId + i;
        if (currentPadId > 15) break;
        
        try {
          await invoke("load_audio", { path: validPaths[i], padId: currentPadId });
          loadedIndices.push(currentPadId);
        } catch (err) {
          console.error(`Error loading audio for pad ${currentPadId}:`, err);
          if (statusDisplay) typeText(statusDisplay, "LOAD ERROR", 10);
        }
      }

      if (loadedIndices.length > 0) {
        const endPadId = loadedIndices[loadedIndices.length - 1];
        if (statusDisplay) {
          typeText(statusDisplay, formatDropMessage(loadedIndices.length, startPadId, endPadId), 10);
        }
        playCascadeAnimation(loadedIndices);
        
        setTimeout(() => {
          if (statusDisplay && statusDisplay.innerText.startsWith("LOADED")) {
            statusDisplay.innerText = "";
          }
        }, 2000);
      }
    }
  });

  // UI elements for Effects
  const effectSelectorBtn = document.getElementById("effect-selector-btn");
  const effectSelectorGrid = document.getElementById("effect-selector-grid");
  const effectOptions = document.querySelectorAll<HTMLElement>(".effect-option");
  const knobs = document.querySelectorAll<HTMLElement>(".knob");

  // Toggle effect selector
  effectSelectorBtn?.addEventListener("click", () => {
    effectSelectorGrid?.classList.toggle("hidden");
  });

  // Handle effect selection
  effectOptions.forEach(opt => {
    opt.addEventListener("click", async () => {
      const effectName = opt.dataset.effect || "None";
      
      // Update UI
      effectOptions.forEach(o => o.classList.remove("active"));
      if (effectName !== "None") {
        opt.classList.add("active");
        if (effectSelectorBtn) {
          effectSelectorBtn.innerHTML = `FX: ${opt.innerText}`;
          // Make sure the arrow spans correctly with flex
          const textNode = document.createTextNode("");
          effectSelectorBtn.appendChild(textNode); 
        }
      } else {
        if (effectSelectorBtn) effectSelectorBtn.innerHTML = `FX: None`;
      }
      
      effectSelectorGrid?.classList.add("hidden");
      
      // Send to Rust
      try {
        if (effectName === "None") {
          await invoke("remove_bus_effect", { bus: currentEditBus, slot: 0 });
        } else {
          await invoke("set_bus_effect", { bus: currentEditBus, slot: 0, effect: effectName });
        }
        if (statusDisplay) typeText(statusDisplay, `FX: ${effectName.toUpperCase()}`, 10);
      } catch (err) {
        console.error("Error setting effect:", err);
      }
    });
  });

  // Close dropdown if clicking outside
  document.addEventListener("click", (e) => {
    if (effectSelectorBtn && effectSelectorGrid && !effectSelectorBtn.contains(e.target as Node) && !effectSelectorGrid.contains(e.target as Node)) {
      effectSelectorGrid.classList.add("hidden");
    }
  });

  // Handle knobs
  knobs.forEach(knob => {
    let isDragging = false;
    let startY = 0;
    let currentVal = 0; // 0.0 to 1.0

    knob.addEventListener("mousedown", (e) => {
      isDragging = true;
      startY = e.clientY;
      knob.style.cursor = "grabbing";
    });

    window.addEventListener("mousemove", async (e) => {
      if (!isDragging) return;
      const deltaY = startY - e.clientY;
      startY = e.clientY; // reset for continuous drag
      
      currentVal += deltaY * 0.005; // sensitivity
      currentVal = Math.max(0, Math.min(1, currentVal));
      
      // Visual update: -135deg to +135deg
      const angle = -135 + (currentVal * 270);
      knob.style.transform = `rotate(${angle}deg)`;
      
      // Throttle or send param to rust
      const paramId = parseInt(knob.dataset.param || "0", 10);
      try {
        await invoke("set_effect_param", { bus: currentEditBus, slot: 0, paramId, value: currentVal });
      } catch (err) {
        console.error("Error setting param:", err);
      }
    });

    window.addEventListener("mouseup", () => {
      if (isDragging) {
        isDragging = false;
        knob.style.cursor = "grab";
      }
    });
  });
});
