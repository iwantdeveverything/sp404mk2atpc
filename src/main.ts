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

window.addEventListener("DOMContentLoaded", () => {
  const pads = document.querySelectorAll<HTMLButtonElement>(".pad");
  const uploadBtn = document.getElementById("audio-upload-btn");
  const targetPadDisplay = document.getElementById("target-pad");
  const statusDisplay = document.getElementById("status-display");

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
    animatePad(padId);
    if (statusDisplay) statusDisplay.innerText = `PLAYING PAD ${padId + 1}`;
    try {
      await invoke("trigger_pad", { padId });
    } catch (err) {
      console.error("Error triggering pad:", err);
      if (statusDisplay) statusDisplay.innerText = `ERROR PAD ${padId + 1}`;
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
      const padId = parseInt(pad.dataset.pad || "0", 10);
      currentTargetPad = padId;
      if (targetPadDisplay) targetPadDisplay.innerText = (padId + 1).toString();
      
      pads.forEach(p => p.classList.remove("target"));
      pad.classList.add("target");
      setTimeout(() => pad.classList.remove("target"), 300);
      
      if (statusDisplay) statusDisplay.innerText = `TARGET PAD ${padId + 1}`;
    });
  });

  // Handle keyboard
  window.addEventListener("keydown", (e) => {
    if (e.repeat) return; // Ignore hold
    const padId = keyMap[e.key.toLowerCase()];
    if (padId !== undefined) {
      triggerPad(padId);
    }
  });

  // Handle file load via Tauri dialog plugin
  uploadBtn?.addEventListener("click", async () => {
    try {
      const file = await open({
        multiple: false,
        filters: [{
          name: 'Audio',
          extensions: ['wav', 'mp3']
        }]
      });

      if (!file) return; // User cancelled

      if (statusDisplay) statusDisplay.innerText = "LOADING...";
      
      await invoke("load_audio", { path: file, padId: currentTargetPad });
      if (statusDisplay) statusDisplay.innerText = `LOADED PAD ${currentTargetPad + 1}`;
    } catch (err) {
      console.error("Error loading audio:", err);
      if (statusDisplay) statusDisplay.innerText = "LOAD ERROR";
    }
  });

  // Handle native drag and drop
  getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'over') {
      const { position } = event.payload;
      const el = document.elementFromPoint(position.x, position.y);
      const padEl = el?.closest('.pad');
      pads.forEach(p => p.classList.toggle('drag-hover', p === padEl));
    } else if (event.payload.type === 'leave' || event.payload.type === 'cancel') {
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
        if (statusDisplay) statusDisplay.innerText = "INVALID FORMAT";
        if (validPaths.length === 0) return;
      }

      if (statusDisplay) statusDisplay.innerText = "LOADING...";

      for (let i = 0; i < validPaths.length; i++) {
        const currentPadId = startPadId + i;
        if (currentPadId > 15) break;
        
        try {
          await invoke("load_audio", { path: validPaths[i], padId: currentPadId });
          if (statusDisplay) statusDisplay.innerText = `LOADED PAD ${currentPadId + 1}`;
        } catch (err) {
          console.error(`Error loading audio for pad ${currentPadId}:`, err);
          if (statusDisplay) statusDisplay.innerText = "LOAD ERROR";
        }
      }
    }
  });
});
